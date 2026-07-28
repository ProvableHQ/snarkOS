// Copyright (c) 2019-2026 Provable Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:

// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shared plumbing for handshakes based on the [Noise-XX] pattern.
//!
//! The Noise static keys are generated per connection: they exist solely to give the pattern a
//! channel to bind to, and are never persisted. Node identity remains the Aleo account key, which
//! is bound to the session by signing the running Noise handshake hash - see [`binding_message`].
//!
//! Note that the transport keys a completed session yields are only used to carry the last
//! handshake message, and are then dropped.
//!
//! [Noise-XX]: https://noiseprotocol.org/noise.html#interactive-handshake-patterns-fundamental

use bytes::BytesMut;
use snow::{Builder, HandshakeState, TransportState, params::NoiseParams};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_util::codec::Framed;

/// The Noise handshake pattern used by snarkOS.
pub const NOISE_PARAMS: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";

/// The prefix that marks a stream as speaking the Noise handshake.
///
/// A peer that only knows the legacy handshake reads these bytes as the little-endian `u32` length
/// prefix of the first frame; the value is `0xFF00_1EAE` (~4.3 GiB), which is far beyond the 1 MiB
/// frame limit its handshake codec allows, so it rejects the connection immediately rather than
/// stalling or misparsing it.
///
/// The prefix travels ahead of the Noise stream and so is not a Noise message, but it is fed to the
/// pattern as its prologue, which mixes it into the handshake hash on both sides. It is therefore
/// covered by the signatures that bind the two identities to the session, exactly like the payloads
/// are - if it is ever tampered with in flight, the two sides derive different hashes and the
/// handshake fails. Nothing that precedes the pattern should be left out of the prologue: the moment
/// the preamble carries anything negotiable, an unauthenticated one is a downgrade attack.
pub const NOISE_MAGIC: [u8; 4] = [0xAE, 0x1E, 0x00, 0xFF];

/// The maximum length of a single Noise message, as mandated by the specification.
pub const MAX_NOISE_MSG_LEN: usize = 65535;

/// The length of the little-endian `u32` that prefixes each Noise message on the wire.
///
/// The specification's own convention is a two-byte big-endian prefix; four little-endian bytes are
/// used instead so that the marker preceding the stream can be chosen to look like an impossible
/// frame length to a peer that speaks the legacy handshake. See [`NOISE_MAGIC`].
const LENGTH_PREFIX_LEN: usize = 4;

/// An upper bound on the number of bytes a Noise message can add on top of its payload: an
/// ephemeral public key, an encrypted static public key with its AEAD tag, and a tag for the payload
/// itself.
///
/// No single message of the pattern carries all of them - the largest is the second, at `e, ee, s,
/// es` - so this is deliberately an upper bound rather than an exact figure. It is used both to
/// reject payloads that could not fit in a message and to size the buffer a message is written into,
/// so it must not be an underestimate for *any* message: the second message is the one that makes
/// the ephemeral key term necessary.
const NOISE_OVERHEAD: usize = DH_LEN + (DH_LEN + TAG_LEN) + TAG_LEN;

/// The length of the AEAD tag of the cipher function in [`NOISE_PARAMS`].
const TAG_LEN: usize = 16;

/// The length of the handshake hash, i.e. the digest length of the hash function in [`NOISE_PARAMS`].
pub const HANDSHAKE_HASH_LEN: usize = 32;

/// The length of a public or private key of the Diffie-Hellman function in [`NOISE_PARAMS`].
const DH_LEN: usize = 32;

/// The offset of the payload within the pattern's first message.
///
/// That message is `e`: the initiator's ephemeral public key followed by the payload, neither of
/// them encrypted, since no key has been established yet. The payload can therefore be read - though
/// emphatically not trusted - before any elliptic curve operation is performed, which is what lets a
/// responder turn a peer away for free; see [`PendingSession`].
///
/// A unit test pins this against a message the pattern actually produced, so that a change to
/// [`NOISE_PARAMS`] cannot invalidate it silently.
const FIRST_MESSAGE_PAYLOAD_OFFSET: usize = DH_LEN;

/// The side of a Noise session; note that this is the side of the *handshake*, which for all
/// current callers coincides with the side of the underlying TCP connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Initiator,
    Responder,
}

impl Role {
    /// The role tag included in the signed binding message.
    const fn tag(&self) -> &'static [u8] {
        match self {
            Self::Initiator => b"|initiator",
            Self::Responder => b"|responder",
        }
    }
}

/// Returns the message that a party must sign with its Aleo account key in order to bind that
/// identity to the Noise session identified by `handshake_hash`.
///
/// The handshake hash commits to every key and payload exchanged so far, so a signature over it is
/// only valid for the one session it was produced in. This is what makes relaying impossible: an
/// attacker that terminates two separate Noise sessions and forwards the payloads between them
/// derives a different handshake hash on each side, so neither signature verifies.
///
/// `domain` separates the handshakes of different subprotocols (e.g. the BFT gateway and the
/// router), which a validator runs concurrently under the same Aleo key.
pub fn binding_message(domain: &[u8], role: Role, handshake_hash: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(domain.len() + role.tag().len() + handshake_hash.len());
    message.extend_from_slice(domain);
    message.extend_from_slice(role.tag());
    message.extend_from_slice(handshake_hash);
    message
}

/// The handshake protocol a connection is speaking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandshakeProtocol {
    /// The Noise-based handshake, identified by the [`NOISE_MAGIC`] prefix.
    Noise,
    /// The legacy challenge-response handshake.
    Legacy,
}

/// Announces to the peer that this side is about to perform a Noise handshake.
///
/// The marker is also the pattern's prologue, so it does not need to be authenticated separately;
/// see [`NOISE_MAGIC`].
pub async fn write_noise_magic<S: AsyncWrite + Unpin>(stream: &mut S) -> io::Result<()> {
    stream.write_all(&NOISE_MAGIC).await
}

/// Determines which handshake protocol the peer is speaking by consuming the first
/// [`NOISE_MAGIC`]-sized chunk of the stream.
///
/// Alongside the protocol, this returns the bytes that need to be handed back to the caller's
/// decoder: empty for [`HandshakeProtocol::Noise`], since the magic is not part of the Noise
/// stream, and the consumed prefix for [`HandshakeProtocol::Legacy`], where it is the beginning of
/// the peer's first frame. Use [`prepare_framed`] to feed it back into a codec.
///
/// Note: this reads rather than peeks, as a peek is free to return fewer bytes than requested.
pub async fn detect_handshake_protocol<S: AsyncRead + Unpin>(
    stream: &mut S,
) -> io::Result<(HandshakeProtocol, BytesMut)> {
    let mut prefix = [0u8; NOISE_MAGIC.len()];
    stream.read_exact(&mut prefix).await?;

    if prefix == NOISE_MAGIC {
        Ok((HandshakeProtocol::Noise, BytesMut::new()))
    } else {
        Ok((HandshakeProtocol::Legacy, BytesMut::from(&prefix[..])))
    }
}

/// Frames the given stream with the given codec, pre-populating the read buffer with bytes that
/// were consumed from the stream before it was framed.
///
/// This exists only so that the legacy handshake can be handed back the prefix that
/// [`detect_handshake_protocol`] took from it; the Noise handshake reads its messages exactly and has
/// no codec to seed. It goes away with the legacy path.
pub fn prepare_framed<S: AsyncRead + AsyncWrite, C>(stream: S, codec: C, read_buf: &[u8]) -> Framed<S, C> {
    let mut framed = Framed::new(stream, codec);
    framed.read_buffer_mut().extend_from_slice(read_buf);
    framed
}

/// Reads one length-prefixed Noise message from the given stream.
///
/// Both reads are exact, so this consumes the message and not a single byte more. That is what lets
/// the stream be handed on to a reader with a codec of its own once the handshake is done: there is
/// never anything buffered here for that reader to miss. Note that a buffering codec could not offer
/// the same guarantee, as it reads whatever the socket has available.
async fn read_message<S: AsyncRead + Unpin>(stream: &mut S) -> io::Result<Vec<u8>> {
    let mut length = [0u8; LENGTH_PREFIX_LEN];
    stream.read_exact(&mut length).await?;

    // Bound the length before it is used to allocate; the specification caps a Noise message at
    // `MAX_NOISE_MSG_LEN`, so anything larger is a protocol violation rather than a big message.
    let length = u32::from_le_bytes(length) as usize;
    if length > MAX_NOISE_MSG_LEN {
        return Err(invalid_data(format!("the Noise message is too large ({length} bytes)")));
    }

    let mut message = vec![0u8; length];
    stream.read_exact(&mut message).await?;

    Ok(message)
}

/// Writes one length-prefixed Noise message to the given stream.
///
/// The prefix and the body go out in a single write, so that emitting a message costs one syscall
/// rather than two and the peer sees the two halves arrive together.
async fn write_message<S: AsyncWrite + Unpin>(stream: &mut S, message: &[u8]) -> io::Result<()> {
    // `NoiseSession::send` already bounds its payload so that the message the pattern produces cannot
    // reach this; the check is repeated here so that the cast below cannot silently truncate.
    if message.len() > MAX_NOISE_MSG_LEN {
        return Err(invalid_data(format!("the Noise message is too large ({} bytes)", message.len())));
    }

    let mut framed = Vec::with_capacity(LENGTH_PREFIX_LEN + message.len());
    framed.extend_from_slice(&(message.len() as u32).to_le_bytes());
    framed.extend_from_slice(message);

    stream.write_all(&framed).await?;
    stream.flush().await
}

/// Returns a builder for the pattern in [`NOISE_PARAMS`], with [`NOISE_MAGIC`] as its prologue.
fn builder<'a>() -> io::Result<Builder<'a>> {
    // This is a compile-time constant, so a parsing failure is a bug rather than a runtime condition.
    let params: NoiseParams = NOISE_PARAMS.parse().expect("the Noise parameters should be valid");
    // The prologue brings the marker that precedes the pattern under the handshake hash; see
    // `NOISE_MAGIC`.
    Builder::new(params).prologue(NOISE_MAGIC.as_slice()).map_err(invalid_data)
}

/// Builds the handshake state for the given role, with a fresh static keypair.
fn build_state(role: Role) -> io::Result<HandshakeState> {
    // Any 32 bytes are a valid X25519 private key, as the scalar is clamped where it is used, so the
    // key is taken straight from the RNG. `Builder::generate_keypair` would derive the public key
    // here and the builder would derive it again when the state is built, spending a scalar
    // multiplication that nothing reads.
    let private_key: [u8; DH_LEN] = rand::random();
    let builder = builder()?.local_private_key(&private_key).map_err(invalid_data)?;

    match role {
        Role::Initiator => builder.build_initiator(),
        Role::Responder => builder.build_responder(),
    }
    .map_err(invalid_data)
}

fn invalid_data<E: std::fmt::Display>(err: E) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err.to_string())
}

enum SessionState {
    Handshake(Box<HandshakeState>),
    Transport(Box<TransportState>),
}

/// A stream whose first Noise message has been read but not yet processed.
///
/// This exists so that a responder can act on the initiator's cleartext payload *before* deriving
/// any keys. Everything up to [`PendingSession::into_session`] is reading and parsing, so a peer that
/// is going to be turned away - one that is not an authorized validator, say - costs no elliptic
/// curve operations at all. Under a distributed flood of connection attempts, that is the difference
/// between paying for five scalar multiplications per rejected peer and paying for none.
pub struct PendingSession<S> {
    stream: S,
    first_message: Vec<u8>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> PendingSession<S> {
    /// Reads the pattern's first message from the given stream.
    ///
    /// The caller is responsible for the [`NOISE_MAGIC`] prefix, which must already have been
    /// consumed with [`detect_handshake_protocol`].
    pub async fn accept(mut stream: S) -> io::Result<Self> {
        let first_message = read_message(&mut stream).await?;

        Ok(Self { stream, first_message })
    }

    /// Returns the payload the initiator sent alongside its ephemeral key.
    ///
    /// It is neither encrypted nor authenticated, so it is a claim and not a fact: anything acted on
    /// here has to be re-checked against the authenticated copy later in the handshake.
    pub fn first_payload(&self) -> io::Result<&[u8]> {
        self.first_message
            .get(FIRST_MESSAGE_PAYLOAD_OFFSET..)
            .ok_or_else(|| invalid_data("the first Noise message is too short to carry a payload"))
    }

    /// Derives the responder's keys and processes the first message.
    pub fn into_session(self) -> io::Result<NoiseSession<S>> {
        let Self { stream, first_message } = self;
        let mut session =
            NoiseSession { stream, state: SessionState::Handshake(Box::new(build_state(Role::Responder)?)) };
        // The payload was already exposed by `first_payload`, so it is discarded here; processing the
        // message is what advances the pattern.
        session.decrypt(&first_message)?;

        Ok(session)
    }
}

/// A Noise-XX session over a stream.
///
/// Payloads are exchanged with [`NoiseSession::send`] and [`NoiseSession::recv`]; the session moves
/// from the handshake phase to the transport phase via [`NoiseSession::into_transport_mode`], which
/// may only be called once the pattern's three messages have been exchanged.
pub struct NoiseSession<S> {
    stream: S,
    state: SessionState,
}

impl<S: AsyncRead + AsyncWrite + Unpin> NoiseSession<S> {
    /// Creates a Noise session over the given stream.
    ///
    /// The caller is responsible for the [`NOISE_MAGIC`] prefix: the initiator must have sent it
    /// with [`write_noise_magic`], and the responder must have consumed it with
    /// [`detect_handshake_protocol`].
    /// Note that a responder should generally use [`PendingSession::accept`] instead, so that it can
    /// inspect the initiator's cleartext payload before committing to any key derivation.
    pub fn new(stream: S, role: Role) -> io::Result<Self> {
        Ok(Self { stream, state: SessionState::Handshake(Box::new(build_state(role)?)) })
    }

    /// Encrypts the given payload and sends it to the peer.
    pub async fn send(&mut self, payload: &[u8]) -> io::Result<()> {
        if payload.len() + NOISE_OVERHEAD > MAX_NOISE_MSG_LEN {
            return Err(invalid_data(format!("the handshake payload is too large ({} bytes)", payload.len())));
        }

        // The message cannot exceed the payload plus what the pattern adds to it, so there is no
        // reason to reach for the maximum message length here.
        let mut buffer = vec![0u8; payload.len() + NOISE_OVERHEAD];
        let len = match self.state {
            SessionState::Handshake(ref mut state) => state.write_message(payload, &mut buffer),
            SessionState::Transport(ref mut state) => state.write_message(payload, &mut buffer),
        }
        .map_err(invalid_data)?;
        buffer.truncate(len);

        write_message(&mut self.stream, &buffer).await
    }

    /// Receives a message from the peer and returns its decrypted payload.
    pub async fn recv(&mut self) -> io::Result<Vec<u8>> {
        let message = read_message(&mut self.stream).await?;

        self.decrypt(&message)
    }

    /// Processes an already-received message and returns its decrypted payload.
    fn decrypt(&mut self, message: &[u8]) -> io::Result<Vec<u8>> {
        // A payload is never longer than the message that carried it, which `read_message` has
        // already capped at `MAX_NOISE_MSG_LEN`.
        let mut buffer = vec![0u8; message.len()];
        let len = match self.state {
            SessionState::Handshake(ref mut state) => state.read_message(message, &mut buffer),
            SessionState::Transport(ref mut state) => state.read_message(message, &mut buffer),
        }
        .map_err(invalid_data)?;
        buffer.truncate(len);

        Ok(buffer)
    }

    /// Returns the current handshake hash, which commits to every key and payload exchanged so
    /// far; see [`binding_message`].
    ///
    /// Both sides derive the same value after processing the same number of messages. It can only
    /// be read during the handshake phase, so callers that need it must capture it before calling
    /// [`NoiseSession::into_transport_mode`].
    pub fn handshake_hash(&self) -> io::Result<[u8; HANDSHAKE_HASH_LEN]> {
        let SessionState::Handshake(ref state) = self.state else {
            return Err(invalid_data("the Noise handshake hash is only available during the handshake"));
        };

        // The hash length is determined by the hash function in `NOISE_PARAMS`.
        Ok(state.get_handshake_hash().try_into().expect("the Noise handshake hash should be 32 bytes long"))
    }

    /// Transitions the session from the handshake phase to the transport phase, which the pattern
    /// only permits once its three messages have been exchanged.
    pub fn into_transport_mode(self) -> io::Result<Self> {
        let Self { stream, state } = self;

        let SessionState::Handshake(handshake_state) = state else {
            return Err(invalid_data("the Noise session is already in transport mode"));
        };
        let transport_state = handshake_state.into_transport_mode().map_err(invalid_data)?;

        Ok(Self { stream, state: SessionState::Transport(Box::new(transport_state)) })
    }

    /// Recovers the underlying stream.
    ///
    /// Nothing is lost in the process: the session reads its messages exactly, so anything the peer
    /// sent past the end of the handshake is still on the stream for whoever takes it next.
    pub fn into_inner(self) -> S {
        self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::{DuplexStream, duplex};

    /// Runs the three messages of the XX pattern, returning both sessions and the handshake hashes
    /// each side derived after messages 2 and 3.
    async fn perform_xx_handshake(
        payloads: [&[u8]; 3],
    ) -> (NoiseSession<DuplexStream>, NoiseSession<DuplexStream>, [[u8; 32]; 4]) {
        let (initiator_stream, responder_stream) = duplex(1024);
        let mut initiator = NoiseSession::new(initiator_stream, Role::Initiator).unwrap();

        // -> e
        initiator.send(payloads[0]).await.unwrap();
        let pending = PendingSession::accept(responder_stream).await.unwrap();
        assert_eq!(pending.first_payload().unwrap(), payloads[0]);
        let mut responder = pending.into_session().unwrap();

        // <- e, ee, s, es
        responder.send(payloads[1]).await.unwrap();
        assert_eq!(initiator.recv().await.unwrap(), payloads[1]);
        let (initiator_h2, responder_h2) = (initiator.handshake_hash().unwrap(), responder.handshake_hash().unwrap());

        // -> s, se
        initiator.send(payloads[2]).await.unwrap();
        assert_eq!(responder.recv().await.unwrap(), payloads[2]);
        let (initiator_h3, responder_h3) = (initiator.handshake_hash().unwrap(), responder.handshake_hash().unwrap());

        (initiator, responder, [initiator_h2, responder_h2, initiator_h3, responder_h3])
    }

    #[tokio::test]
    async fn xx_handshake_completes_and_agrees_on_the_handshake_hash() {
        let (initiator, responder, [initiator_h2, responder_h2, initiator_h3, responder_h3]) =
            perform_xx_handshake([b"hint", b"responder info", b"initiator info"]).await;

        // Both sides derive the same binding value at both points of the handshake.
        assert_eq!(initiator_h2, responder_h2);
        assert_eq!(initiator_h3, responder_h3);
        // The hash keeps evolving, so the two binding values are distinct.
        assert_ne!(initiator_h2, initiator_h3);

        // The fourth message, carrying the responder's proof, is a transport message.
        let mut initiator = initiator.into_transport_mode().unwrap();
        let mut responder = responder.into_transport_mode().unwrap();
        responder.send(b"responder proof").await.unwrap();
        assert_eq!(initiator.recv().await.unwrap(), b"responder proof");
    }

    #[tokio::test]
    async fn handshake_hashes_differ_between_sessions() {
        let (_, _, first) = perform_xx_handshake([b"", b"", b""]).await;
        let (_, _, second) = perform_xx_handshake([b"", b"", b""]).await;

        // The ephemeral keys make every session's binding value unique, which is what prevents a
        // signature over it from being relayed into another session.
        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn tampering_with_a_handshake_message_is_detected() {
        let (mut initiator_stream, mut responder_stream) = duplex(1024);
        let mut initiator = NoiseSession::new(&mut initiator_stream, Role::Initiator).unwrap();

        initiator.send(b"hint").await.unwrap();
        let mut responder = PendingSession::accept(&mut responder_stream).await.unwrap().into_session().unwrap();

        // Flip a bit in the encrypted payload of the second message.
        let mut buffer = vec![0u8; MAX_NOISE_MSG_LEN];
        let SessionState::Handshake(ref mut state) = responder.state else { unreachable!() };
        let len = state.write_message(b"responder info", &mut buffer).unwrap();
        buffer.truncate(len);
        *buffer.last_mut().unwrap() ^= 1;
        write_message(&mut responder.stream, &buffer).await.unwrap();

        assert!(initiator.recv().await.is_err());
    }

    #[tokio::test]
    async fn a_first_message_payload_is_readable_before_any_keys_are_derived() {
        let (initiator_stream, responder_stream) = duplex(1024);
        let mut initiator = NoiseSession::new(initiator_stream, Role::Initiator).unwrap();
        initiator.send(b"a cleartext hint").await.unwrap();

        // This is what pins `FIRST_MESSAGE_PAYLOAD_OFFSET` to what the pattern actually produces: if
        // `NOISE_PARAMS` ever changed its Diffie-Hellman function, or the pattern gained a
        // pre-message, the offset would be wrong and this assertion would catch it.
        let pending = PendingSession::accept(responder_stream).await.unwrap();
        assert_eq!(pending.first_payload().unwrap(), b"a cleartext hint");

        // The same message must still drive the handshake once the keys exist.
        let mut responder = pending.into_session().unwrap();
        responder.send(b"responder info").await.unwrap();
        assert_eq!(initiator.recv().await.unwrap(), b"responder info");
    }

    #[tokio::test]
    async fn tampering_with_the_cleartext_first_payload_is_detected() {
        // The attacker sits on both wires, so that it can rewrite the first message in flight.
        let (initiator_stream, mut initiator_wire) = duplex(1024);
        let (mut responder_wire, responder_stream) = duplex(1024);

        let mut initiator = NoiseSession::new(initiator_stream, Role::Initiator).unwrap();
        initiator.send(b"the original hint").await.unwrap();

        // Flip a bit in the payload, which the pattern's first message carries in the clear.
        let mut message = read_message(&mut initiator_wire).await.unwrap();
        message[FIRST_MESSAGE_PAYLOAD_OFFSET] ^= 1;
        write_message(&mut responder_wire, &message).await.unwrap();

        // The responder reads the rewritten payload quite happily - it is a claim, not a fact.
        let pending = PendingSession::accept(responder_stream).await.unwrap();
        assert_eq!(pending.first_payload().unwrap(), b"uhe original hint");
        let mut responder = pending.into_session().unwrap();

        // But the payload was mixed into its handshake hash, which is the associated data of every
        // encryption that follows, so its reply cannot be decrypted by the initiator. This is what
        // lets the responder act on the cleartext hint and have the result stand.
        responder.send(b"responder info").await.unwrap();
        let reply = read_message(&mut responder_wire).await.unwrap();
        write_message(&mut initiator_wire, &reply).await.unwrap();

        assert!(initiator.recv().await.is_err());
    }

    #[tokio::test]
    async fn a_mismatched_prologue_fails_the_handshake() {
        // A peer that folds a different marker into the pattern - which is what a marker tampered
        // with in flight amounts to - cannot complete the handshake.
        let params: NoiseParams = NOISE_PARAMS.parse().unwrap();
        let mut odd_one_out = Builder::new(params)
            .prologue(b"a different marker")
            .unwrap()
            .local_private_key(&[0u8; DH_LEN])
            .unwrap()
            .build_initiator()
            .unwrap();

        let (mut initiator_stream, responder_stream) = duplex(1024);

        let mut buffer = vec![0u8; MAX_NOISE_MSG_LEN];
        let len = odd_one_out.write_message(b"hint", &mut buffer).unwrap();
        buffer.truncate(len);
        write_message(&mut initiator_stream, &buffer).await.unwrap();

        // The responder accepts the first message, which carries nothing it could verify, and its
        // reply is rejected in turn - exactly as with a tampered payload.
        let mut responder = PendingSession::accept(responder_stream).await.unwrap().into_session().unwrap();
        responder.send(b"responder info").await.unwrap();

        let reply = read_message(&mut initiator_stream).await.unwrap();
        assert!(odd_one_out.read_message(&reply, &mut vec![0u8; MAX_NOISE_MSG_LEN]).is_err());
    }

    #[tokio::test]
    async fn a_session_leaves_bytes_that_follow_a_message_on_the_stream() {
        let (mut initiator_stream, responder_stream) = duplex(1024);
        let mut initiator = NoiseSession::new(&mut initiator_stream, Role::Initiator).unwrap();
        initiator.send(b"hint").await.unwrap();

        // Whatever the peer pipelines behind a handshake message. Dropping the session first is only
        // to release the borrow; the bytes are written to the same stream either way.
        drop(initiator);
        initiator_stream.write_all(b"pipelined").await.unwrap();

        let pending = PendingSession::accept(responder_stream).await.unwrap();
        assert_eq!(pending.first_payload().unwrap(), b"hint");
        let responder = pending.into_session().unwrap();

        // The session read its message and not a byte further, so the trailing bytes are still there
        // for whoever takes the stream next - which is what lets the handshake hand a bare stream to
        // a reader that builds a codec of its own.
        let mut stream = responder.into_inner();
        let mut trailing = [0u8; 9];
        stream.read_exact(&mut trailing).await.unwrap();
        assert_eq!(&trailing, b"pipelined");
    }

    #[tokio::test]
    async fn a_truncated_first_message_is_rejected() {
        let (mut initiator_stream, responder_stream) = duplex(1024);

        // Shorter than the ephemeral key the pattern's first message must begin with.
        write_message(&mut initiator_stream, &[0u8; DH_LEN - 1]).await.unwrap();

        let pending = PendingSession::accept(responder_stream).await.unwrap();
        assert!(pending.first_payload().is_err());
    }

    #[tokio::test]
    async fn an_oversized_message_length_is_rejected_before_it_is_allocated() {
        let (mut initiator_stream, mut responder_stream) = duplex(1024);

        // A length prefix beyond what the specification permits, and no body behind it.
        initiator_stream.write_all(&(MAX_NOISE_MSG_LEN as u32 + 1).to_le_bytes()).await.unwrap();

        let error = read_message(&mut responder_stream).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn the_binding_message_is_domain_separated() {
        let hash = [7u8; HANDSHAKE_HASH_LEN];

        // A signature is only valid for one role of one subprotocol.
        assert_ne!(binding_message(b"bft", Role::Initiator, &hash), binding_message(b"bft", Role::Responder, &hash));
        assert_ne!(binding_message(b"bft", Role::Initiator, &hash), binding_message(b"router", Role::Initiator, &hash));
    }

    #[tokio::test]
    async fn the_noise_magic_is_detected() {
        let (mut initiator_stream, mut responder_stream) = duplex(1024);

        write_noise_magic(&mut initiator_stream).await.unwrap();
        let (protocol, leftover) = detect_handshake_protocol(&mut responder_stream).await.unwrap();
        assert_eq!(protocol, HandshakeProtocol::Noise);
        assert!(leftover.is_empty());
    }

    #[tokio::test]
    async fn a_legacy_prefix_is_detected_and_returned() {
        let (mut initiator_stream, mut responder_stream) = duplex(1024);

        // The length prefix of a legacy `ChallengeRequest` frame.
        let legacy_prefix = 87u32.to_le_bytes();
        initiator_stream.write_all(&legacy_prefix).await.unwrap();

        let (protocol, leftover) = detect_handshake_protocol(&mut responder_stream).await.unwrap();
        assert_eq!(protocol, HandshakeProtocol::Legacy);
        assert_eq!(&leftover[..], &legacy_prefix[..]);
    }

    #[test]
    fn the_noise_magic_is_an_invalid_legacy_frame_length() {
        // A legacy peer must reject the magic outright instead of waiting for a frame that will
        // never arrive; its handshake codecs cap frames at 1 MiB.
        const MAX_LEGACY_HANDSHAKE_FRAME_LEN: u32 = 1024 * 1024;
        assert!(u32::from_le_bytes(NOISE_MAGIC) > MAX_LEGACY_HANDSHAKE_FRAME_LEN);
    }
}
