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

#![forbid(unsafe_code)]

#[macro_use]
extern crate tracing;

mod helpers;
// Imports custom `Path` type, to be used instead of `axum`'s.
pub use helpers::*;

mod routes;

mod version;

use snarkos_node_cdn::CdnBlockSync;
use snarkos_node_consensus::Consensus;
use snarkos_node_router::{
    Routing,
    messages::{Message, UnconfirmedTransaction},
};
use snarkos_node_sync::BlockSync;
use snarkvm::{
    console::{program::ProgramID, types::Field},
    ledger::narwhal::Data,
    prelude::{Ledger, Network, VM, cfg_into_iter, store::ConsensusStorage},
};

use anyhow::{Context, Result, anyhow};
use axum::{
    body::Body,
    extract::{ConnectInfo, DefaultBodyLimit, Query, State},
    http::{Method, Request, StatusCode, header::CONTENT_TYPE},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::response::ErasedJson;
#[cfg(feature = "locktick")]
use locktick::parking_lot::Mutex;
use lru::LruCache;
#[cfg(not(feature = "locktick"))]
use parking_lot::Mutex;
use std::{net::SocketAddr, num::NonZeroUsize, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Semaphore, task::JoinHandle};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::Span;

/// The default port used for the REST API
pub const DEFAULT_REST_PORT: u16 = 3030;

/// The API version prefixes.
pub const API_VERSION_V1: &str = "v1";
pub const API_VERSION_V2: &str = "v2";

/// The capacity of the LRU holding recently requested blocks.
const BLOCK_CACHE_SIZE: usize = 128;

/// The maximum size of a request body, for every endpoint except `POST /transaction/broadcast`.
///
/// Nothing else this node accepts carries a payload that scales with consensus limits, so this
/// stays a fixed value rather than tracking one.
const DEFAULT_BODY_LIMIT: usize = 2 * 768 * 1024; // 1.5 MiB

/// How much larger a transaction's JSON encoding may be than the binary encoding that
/// `MAX_TRANSACTION_SIZE` bounds.
///
/// The two are not in the same units, which is what makes a factor necessary at all. Verifying
/// keys and certificates are rendered as bech32m strings, which pack five bits per character
/// rather than eight, and a deployment's program is rendered as its source text rather than as
/// the parsed form the binary encoding carries.
///
/// This covers a deployment, which is the largest transaction there is: its provable ceiling is
/// about 1.97x and a realistic one about 1.10x, leaving roughly 2.4 MB of headroom. It also
/// covers an ordinary execution, which measures around 1.5x.
///
/// It is deliberately **not** a universal upper bound, and the gap is worth stating plainly.
/// `Plaintext`'s human-readable form pretty-prints nested values with two spaces of indentation
/// per level of depth, and an array type may nest up to `MAX_DATA_DEPTH` (32) levels - so for an
/// execution the ratio grows linearly with nesting depth rather than sitting at a constant. A
/// `[boolean; 512]` measures 1.50x flat, 3.82x at depth 7, and 11.04x at depth 29. An execution
/// whose inputs nest past roughly depth 6 will therefore be rejected here even though the network
/// considers it valid.
///
/// Covering `MAX_DATA_DEPTH` outright would mean a factor of about 12, and a body limit near
/// 27 MB on an endpoint whose concurrency is unbounded (the governor below rate-limits requests
/// per IP, not bytes in flight). That trade was judged not worth making for a shape no real
/// transaction has; raise this, or derive the limit component-wise, if one ever does.
const TRANSACTION_JSON_EXPANSION_FACTOR: usize = 3;

/// The total request-body bytes the node will hold across all in-flight transaction broadcasts.
///
/// `DefaultBodyLimit` bounds one body; nothing bounds their sum. `/transaction/broadcast` is
/// unauthenticated and enabled by default on every client and validator (`0.0.0.0:3030` unless
/// `--norest`), and `GovernorLayer` rate-limits requests per IP, not bytes - so without this the
/// memory an anonymous caller can pin is bounded only by how many connections they can open.
///
/// Nothing upstream can be relied on to do this instead. A proxy, where one exists at all, carries
/// whatever limit its operator configured: the public API accepts 20 MB bodies today, and a node
/// exposed directly has no proxy whatsoever.
///
/// The budget is charged per request by `Content-Length`, so ordinary traffic barely touches it -
/// a few thousand bytes each, tens of thousands of them concurrently. Only genuinely large bodies
/// consume it, which is the traffic worth bounding. It must stay at or above
/// `transaction_body_limit`, or a single maximum-size deployment could never be admitted;
/// `the_budget_admits_a_maximum_size_body` pins that.
const TRANSACTION_BODY_BYTES_IN_FLIGHT: usize = 64 * 1024 * 1024; // 64 MiB

/// The maximum size of a `POST /transaction/broadcast` body.
///
/// Derived from the consensus limit rather than hardcoded: `MAX_TRANSACTION_SIZE` has been raised
/// twice (128 kB, then 768 kB, then 2304 kB at V16), and a body limit that does not track it
/// silently rejects transactions the network itself considers valid - a deployment, which is by
/// far the largest transaction there is, being the case that actually reaches the cap.
fn transaction_body_limit<N: Network>() -> usize {
    N::LATEST_MAX_TRANSACTION_SIZE().saturating_mul(TRANSACTION_JSON_EXPANSION_FACTOR)
}

/// A REST API server for the ledger.
#[derive(Clone)]
pub struct Rest<N: Network, C: ConsensusStorage<N>, R: Routing<N>> {
    /// CDN sync (only if node is using the CDN to sync).
    cdn_sync: Option<Arc<CdnBlockSync>>,
    /// The consensus module.
    consensus: Option<Consensus<N>>,
    /// The ledger.
    ledger: Ledger<N, C>,
    /// The node (routing).
    routing: Arc<R>,
    /// The server handles.
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    /// A reference to BlockSync,
    block_sync: Arc<BlockSync<N>>,
    /// The number of ongoing deploy transaction verifications via REST.
    num_verifying_deploys: Arc<Semaphore>,
    /// The number of ongoing execute transaction verifications via REST.
    num_verifying_executions: Arc<Semaphore>,
    /// The number of ongoing solution verifications via REST.
    num_verifying_solutions: Arc<Semaphore>,
    /// The request-body bytes currently held across in-flight transaction broadcasts, one permit
    /// per byte. See [`TRANSACTION_BODY_BYTES_IN_FLIGHT`].
    transaction_body_bytes: Arc<Semaphore>,
    /// A cache containing recently requested blocks.
    block_cache: Arc<Mutex<LruCache<N::BlockHash, ErasedJson>>>,
}

impl<N: Network, C: 'static + ConsensusStorage<N>, R: Routing<N>> Rest<N, C, R> {
    /// Initializes a new instance of the server.
    pub async fn start(
        rest_ip: SocketAddr,
        rest_rps: u32,
        consensus: Option<Consensus<N>>,
        ledger: Ledger<N, C>,
        routing: Arc<R>,
        cdn_sync: Option<Arc<CdnBlockSync>>,
        block_sync: Arc<BlockSync<N>>,
    ) -> Result<Self> {
        // Initialize the server.
        let mut server = Self {
            consensus,
            ledger,
            routing,
            cdn_sync,
            block_sync,
            handles: Default::default(),
            num_verifying_deploys: Arc::new(Semaphore::new(VM::<N, C>::MAX_PARALLEL_DEPLOY_VERIFICATIONS)),
            num_verifying_executions: Arc::new(Semaphore::new(VM::<N, C>::MAX_PARALLEL_EXECUTE_VERIFICATIONS)),
            num_verifying_solutions: Arc::new(Semaphore::new(N::MAX_SOLUTIONS)),
            transaction_body_bytes: Arc::new(Semaphore::new(TRANSACTION_BODY_BYTES_IN_FLIGHT)),
            block_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(BLOCK_CACHE_SIZE).unwrap()))),
        };
        // Spawn the server.
        server.spawn_server(rest_ip, rest_rps).await?;
        // Return the server.
        Ok(server)
    }
}

impl<N: Network, C: ConsensusStorage<N>, R: Routing<N>> Rest<N, C, R> {
    /// Returns the ledger.
    pub const fn ledger(&self) -> &Ledger<N, C> {
        &self.ledger
    }

    /// Returns the handles.
    pub const fn handles(&self) -> &Arc<Mutex<Vec<JoinHandle<()>>>> {
        &self.handles
    }

    /// Shuts down the REST instance.
    pub fn shut_down(&self) {
        self.handles.lock().iter().for_each(|handle| handle.abort());
    }
}

impl<N: Network, C: ConsensusStorage<N>, R: Routing<N>> Rest<N, C, R> {
    fn build_routes(&self, rest_rps: u32) -> axum::Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
            .allow_headers([CONTENT_TYPE]);

        // Prepare the rate limiting setup.
        let governor_config = Box::new(
            GovernorConfigBuilder::default()
                .per_nanosecond((1_000_000_000 / rest_rps) as u64)
                .burst_size(rest_rps)
                .error_handler(|error| {
                    // Properly return a 429 Too Many Requests error
                    let error_message = error.to_string();
                    let mut response = Response::new(error_message.clone().into());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    if error_message.contains("Too Many Requests") {
                        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                    }
                    response
                })
                .finish()
                .expect("Couldn't set up rate limiting for the REST server!"),
        );

        // Build the JWT auth-protected endpoints. #[cfg] cannot appear inside a method chain, so we
        // build this router as a named binding and conditionally extend it before applying the layer.
        let auth_routes = axum::Router::new()
            .route("/node/address", get(Self::get_node_address))
            .route("/program/{id}/mapping/{name}", get(Self::get_mapping_values))
            .route("/db_backup", post(Self::db_backup));

        // Slipstream plugin management endpoints require auth.
        #[cfg(feature = "slipstream-plugins")]
        let auth_routes = auth_routes
            .route("/slipstream/plugins", get(Self::slipstream_list_plugins).post(Self::slipstream_load_plugin))
            .route(
                "/slipstream/plugins/{name}",
                // TODO: PUT (reload) is not yet implemented.
                axum::routing::delete(Self::slipstream_unload_plugin),
            );

        let routes = axum::Router::new()
            .merge(auth_routes.route_layer(middleware::from_fn(auth_middleware)))

            // All endpoints declared after here are not protected

             // Get ../consensus_version
            .route("/consensus_version", get(Self::get_consensus_version))

            // GET ../block/..
            .route("/block/height/latest", get(Self::get_block_height_latest))
            .route("/block/hash/latest", get(Self::get_block_hash_latest))
            .route("/block/latest", get(Self::get_block_latest))
            .route("/block/{height_or_hash}", get(Self::get_block))
            // The path param here is actually only the height, but the name must match the route
            // above, otherwise there'll be a conflict at runtime.
            .route("/block/{height_or_hash}/header", get(Self::get_block_header))
            .route("/block/{height_or_hash}/transactions", get(Self::get_block_transactions))

            // GET and POST ../transaction/..
            .route("/transaction/{id}", get(Self::get_transaction))
            .route("/transaction/confirmed/{id}", get(Self::get_confirmed_transaction))
            .route("/transaction/unconfirmed/{id}", get(Self::get_unconfirmed_transaction))
            .route("/transaction/rejected/{id}/reason", get(Self::get_transaction_rejection_reason))
            // A transaction body may legitimately be far larger than any other request body, so
            // this route carries its own limit. Being applied to the `MethodRouter` rather than
            // to the `Router`, it sits closer to the handler than the blanket
            // `DEFAULT_BODY_LIMIT` layer below and therefore overrides it for this route alone.
            .route(
                "/transaction/broadcast",
                post(Self::transaction_broadcast)
                    .layer(DefaultBodyLimit::max(transaction_body_limit::<N>()))
                    .layer(middleware::from_fn({
                        // Cloned per `build_routes` call - the default, v1 and v2 prefixes each
                        // get their own layer - but all of them share the one `Arc`, so the budget
                        // is node-wide rather than per-prefix.
                        let budget = self.transaction_body_bytes.clone();
                        move |request, next| {
                            limit_body_bytes_in_flight(
                                budget.clone(),
                                transaction_body_limit::<N>(),
                                request,
                                next,
                            )
                        }
                    })),
            )

            // GET and POST ../solution/..
            .route("/solution/limits/{prover_address}", get(Self::get_solution_limits_for_prover))
            .route("/solution/broadcast", post(Self::solution_broadcast))

            // GET ../find/..
            .route("/find/blockHash/{tx_id}", get(Self::find_block_hash))
            .route("/find/blockHeight/{state_root}", get(Self::find_block_height_from_state_root))
            .route("/find/transactionID/deployment/{program_id}", get(Self::find_latest_transaction_id_from_program_id))
            .route("/find/transactionID/deployment/{program_id}/{edition}", get(Self::find_latest_transaction_id_from_program_id_and_edition))
            .route("/find/transactionID/deployment/{program_id}/{edition}/original", get(Self::find_original_deployment_transaction_id))
            .route("/find/transactionID/deployment/{program_id}/{edition}/{amendment}", get(Self::find_transaction_id_from_program_id_edition_and_amendment))
            .route("/find/transactionID/{transition_id}", get(Self::find_transaction_id_from_transition_id))
            .route("/find/transitionID/{input_or_output_id}", get(Self::find_transition_id))

            // GET ../connections/p2p/.. (with ../peers/.. aliases)
            .route("/peers/count", get(Self::get_peers_count))
            .route("/peers/all", get(Self::get_peers_all))
            .route("/peers/all/metrics", get(Self::get_peers_all_metrics))
            .route("/connections/p2p/count", get(Self::get_peers_count))
            .route("/connections/p2p/all", get(Self::get_peers_all))
            .route("/connections/p2p/all/metrics", get(Self::get_peers_all_metrics))

            // GET ../program/..
            .route("/program/{id}", get(Self::get_program))
            .route("/program/{id}/latest_edition", get(Self::get_latest_program_edition))
            .route("/program/{id}/{edition}", get(Self::get_program_for_edition))
            .route("/program/{id}/mappings", get(Self::get_mapping_names))
            .route("/program/{id}/mapping/{name}/{key}", get(Self::get_mapping_value))
            .route("/program/{id}/amendment_count", get(Self::get_program_amendment_count))
            .route("/program/{id}/{edition}/amendment_count", get(Self::get_program_amendment_count_for_edition))

            // GET ../sync/..
            // Note: keeping ../sync_status for compatibility
            .route("/sync_status", get(Self::get_sync_status))
            .route("/sync/status", get(Self::get_sync_status))
            .route("/sync/peers", get(Self::get_sync_peers))
            .route("/sync/requests", get(Self::get_sync_requests_summary))
            .route("/sync/requests/list", get(Self::get_sync_requests_list))

            // GET misc endpoints.
            .route("/version", get(Self::get_version))
            .route("/blocks", get(Self::get_blocks))
            .route("/height/{hash}", get(Self::get_height))
            .route("/memoryPool/transmissions", get(Self::get_memory_pool_transmissions))
            .route("/memoryPool/solutions", get(Self::get_memory_pool_solutions))
            .route("/memoryPool/transactions", get(Self::get_memory_pool_transactions))
            .route("/statePath/{commitment}", get(Self::get_state_path_for_commitment))
            .route("/statePaths", get(Self::get_state_paths_for_commitments))
            .route("/stateRoot/latest", get(Self::get_state_root_latest))
            .route("/stateRoot/{height}", get(Self::get_state_root))
            .route("/committee/latest", get(Self::get_committee_latest))
            .route("/committee/{height}", get(Self::get_committee))
            .route("/delegators/{validator}", get(Self::get_delegators_for_validator));

        // If the node is a validator, enable the BFT connections endpoints.
        let routes = match self.consensus {
            Some(_) => routes
                .route("/connections/bft/count", get(Self::get_bft_connections_count))
                .route("/connections/bft/all", get(Self::get_bft_connections_all)),
            None => routes,
        };

        // If the node is a validator and `telemetry` features is enabled, enable the additional endpoint.
        #[cfg(feature = "metrics")]
        let routes = match self.consensus {
            Some(_) => routes.route("/validators/participation", get(Self::get_validator_participation_scores)),
            None => routes,
        };

        // Register the view-at-latest-height endpoint (always available, no history required).
        let routes = routes.route("/program/{id}/view/{function}", post(Self::evaluate_view_latest));

        // If the `history` feature is enabled, enable the additional endpoints.
        #[cfg(feature = "history")]
        let routes = routes
            .route("/program/{id}/mapping/{name}/{key}/history/{height}", get(Self::get_history))
            .route("/program/{id}/mapping/{name}/history/{height}", get(Self::get_history_batch))
            .route("/program/{id}/view/{function}/{height}", post(Self::evaluate_view));

        // If the `history-staking-rewards` feature is enabled, enable the additional endpoint.
        #[cfg(feature = "history-staking-rewards")]
        let routes = routes.route("/staking/rewards/{address}/{height}", get(Self::get_staking_reward));

        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                let addr = request
                    .extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ConnectInfo(addr)| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Create a span that includes method, path, and our extracted IP
                tracing::info_span!(
                    "REST",
                    method = %request.method(),
                    uri = %request.uri().path(),
                    addr = %addr,
                )
            })
            .on_request(|_request: &Request<_>, _span: &Span| {
                info!("Received a request");
            })
            .on_response(|_response: &Response<_>, latency: Duration, _span: &Span| {
                info!("Finished request in {:?}", latency);
            });

        routes
            // Pass in `Rest` to make things convenient.
            .with_state(self.clone())
            // Cap the request body size. `/transaction/broadcast` overrides this with its own,
            // larger limit above.
            .layer(DefaultBodyLimit::max(DEFAULT_BODY_LIMIT))
            .layer(GovernorLayer {
                config: governor_config.into(),
            })
            // Enable CORS.
            .layer(cors)
            // Enable tower-http tracing.
            .layer(trace_layer)
    }

    async fn spawn_server(&mut self, rest_ip: SocketAddr, rest_rps: u32) -> Result<()> {
        // Log the REST rate limit per IP.
        debug!("REST rate limit per IP - {rest_rps} RPS");

        // Add the v1 API as default and under "/v1".
        let default_router = axum::Router::new().nest(
            &format!("/{}", N::SHORT_NAME),
            self.build_routes(rest_rps).layer(middleware::map_response(v1_error_middleware)),
        );
        let v1_router = axum::Router::new().nest(
            &format!("/{API_VERSION_V1}/{}", N::SHORT_NAME),
            self.build_routes(rest_rps).layer(middleware::map_response(v1_error_middleware)),
        );

        // Add the v2 API under "/v2".
        let v2_router =
            axum::Router::new().nest(&format!("/{API_VERSION_V2}/{}", N::SHORT_NAME), self.build_routes(rest_rps));

        // Combine all routes.
        let router = default_router.merge(v1_router).merge(v2_router);

        let rest_listener =
            TcpListener::bind(rest_ip).await.with_context(|| "Failed to bind TCP port for REST endpoints")?;

        let handle = tokio::spawn(async move {
            axum::serve(rest_listener, router.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .expect("couldn't start rest server");
        });

        self.handles.lock().push(handle);
        Ok(())
    }
}

/// Converts errors to the old style for the v1 API.
/// The error code will always be 500 and the content a simple string.
/// Charges a request's body against the node-wide [`TRANSACTION_BODY_BYTES_IN_FLIGHT`] budget for
/// as long as it is in flight, rejecting it if the budget is exhausted.
///
/// This runs before the handler, so the permit is held across the body being buffered and parsed -
/// which is the point. The verification semaphores the handler acquires cannot serve here: they
/// are taken after `Json<Transaction<N>>` has already read the whole body into memory, and only
/// when `check_transaction=true`, so they never bound this at all.
///
/// A body with no `Content-Length` (a chunked upload) is charged the full per-request limit, since
/// `DefaultBodyLimit` is then the only thing bounding what may arrive. A declared length above
/// that limit is charged the limit rather than rejected here; `DefaultBodyLimit` rejects it on its
/// own terms, and this layer's job is accounting, not validation.
///
/// A declared length cannot under-state what arrives. hyper holds a non-chunked body to its
/// `Content-Length`, and a chunked one carries no length to under-state - it lands in the case
/// above and is charged the maximum. So the charge is either honest or conservative, never short.
async fn limit_body_bytes_in_flight(
    budget: Arc<Semaphore>,
    max_body_len: usize,
    request: Request<Body>,
    next: middleware::Next,
) -> Response {
    let declared = request
        .headers()
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok());
    let charge = declared.unwrap_or(max_body_len).min(max_body_len);
    // Saturating rather than fallible: a charge above `u32::MAX` cannot be admitted by any budget
    // this constant permits, so asking for the maximum is the same rejection either way.
    let charge = u32::try_from(charge).unwrap_or(u32::MAX);

    let Ok(_permit) = budget.try_acquire_many_owned(charge) else {
        return RestError::too_many_requests(anyhow!(
            "The node is already holding its limit of in-flight request bytes; retry shortly"
        ))
        .into_response();
    };

    // `_permit` is released when it drops at the end of this function, i.e. once the handler has
    // finished with the body.
    next.run(request).await
}

async fn v1_error_middleware(response: Response) -> Response {
    // The status code used by all v1 errors
    const V1_STATUS_CODE: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

    if response.status().is_success() {
        return response;
    }

    // Returns a opaque error instead of panicking.
    let fallback = || {
        let mut response = Response::new(Body::from("Failed to convert error"));
        *response.status_mut() = V1_STATUS_CODE;
        response
    };

    let Ok(bytes) = axum::body::to_bytes(response.into_body(), usize::MAX).await else {
        return fallback();
    };

    // Deserialize REST error so we can convert it to a string
    let Ok(json_err) = serde_json::from_slice::<SerializedRestError>(&bytes) else {
        return fallback();
    };

    let mut message = json_err.message;
    for next in json_err.chain.into_iter() {
        message = format!("{message} — {next}");
    }

    let mut response = Response::new(Body::from(message));

    *response.status_mut() = V1_STATUS_CODE;

    response
}

/// Formats an ID into a truncated identifier (for logging purposes).
pub fn fmt_id(id: impl ToString) -> String {
    let id = id.to_string();
    let mut formatted_id = id.chars().take(16).collect::<String>();
    if id.chars().count() > 16 {
        formatted_id.push_str("..");
    }
    formatted_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use snarkvm::prelude::MainnetV0;
    use tower::ServiceExt; // for `oneshot`

    /// A router carrying the same body-budget layering as the real `/transaction/broadcast` route,
    /// over a handler that merely reads the body. The production route needs a full `Rest`, and so
    /// a ledger, which these do not.
    fn budget_test_app(budget: Arc<Semaphore>, max_body_len: usize) -> Router {
        async fn read_body(body: axum::body::Bytes) -> String {
            body.len().to_string()
        }

        Router::new().route(
            "/transaction/broadcast",
            post(read_body).layer(DefaultBodyLimit::max(max_body_len)).layer(middleware::from_fn(move |req, next| {
                limit_body_bytes_in_flight(budget.clone(), max_body_len, req, next)
            })),
        )
    }

    /// A request that declares its length, as a real client does - so it is charged what it
    /// actually carries rather than the full per-request limit.
    fn sized_post(len: usize) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/transaction/broadcast")
            .header(axum::http::header::CONTENT_LENGTH, len)
            .body(Body::from(vec![0u8; len]))
            .unwrap()
    }

    /// The limit has to clear the JSON encoding of the largest transaction there is - a
    /// deployment - not merely the binary size the consensus limit bounds. Derived from snarkVM's
    /// own constants rather than asserted as a multiple of itself, so it fails if the factor is
    /// cut, if `MAX_PROGRAM_SIZE` grows relative to `MAX_TRANSACTION_SIZE`, or if a program is
    /// allowed more functions or records.
    ///
    /// This is what the hardcoded `2 * 768 * 1024` failed: derived from a `MAX_TRANSACTION_SIZE`
    /// of 768 kB, it stayed put when V16 raised that to 2304 kB, leaving valid deployments
    /// rejected as `JsonRejection::BytesRejection` - a 400, rewritten to 500 on the v1 prefixes -
    /// before the handler's own size check could report anything better.
    #[test]
    fn transaction_body_limit_covers_a_maximum_size_deployment_in_json() {
        /// A bech32m string carries five bits per character rather than eight, plus a
        /// human-readable prefix and a six-character checksum; 32 characters covers both.
        const fn bech32_len(bytes: usize) -> usize {
            bytes.div_ceil(5) * 8 + 32
        }
        /// A verifying key and its certificate, in bytes, per snarkVM's own sizing comment on
        /// `MAX_TRANSACTION_SIZE`.
        const VERIFYING_KEY_LEN: usize = 673;
        const CERTIFICATE_LEN: usize = 58;
        /// Slack per entry for its identifier, quoting, and separators.
        const ENTRY_PUNCTUATION: usize = 64;

        // The program is rendered as its source text, which `MAX_PROGRAM_SIZE` bounds directly.
        // Doubling it bounds JSON string escaping, since every escape this can produce is two
        // characters.
        let program = MainnetV0::LATEST_MAX_PROGRAM_SIZE() * 2;
        // A verifying key and certificate for every function and every record.
        let entries = MainnetV0::MAX_FUNCTIONS + MainnetV0::MAX_RECORDS;
        let keys = entries * (bech32_len(VERIFYING_KEY_LEN) + bech32_len(CERTIFICATE_LEN) + ENTRY_PUNCTUATION);
        // The fee transition, identifiers, and the surrounding object.
        let remainder = MainnetV0::LATEST_MAX_TRANSACTION_SIZE() - MainnetV0::LATEST_MAX_PROGRAM_SIZE();

        let json_upper_bound = program + keys + remainder;
        assert!(
            transaction_body_limit::<MainnetV0>() >= json_upper_bound,
            "body limit {} is below the JSON bound for a maximum-size deployment ({json_upper_bound})",
            transaction_body_limit::<MainnetV0>()
        );
    }

    /// The budget must admit the largest body the route accepts, or a maximum-size deployment
    /// could never be broadcast at all - the budget would have replaced the cap this PR fixes with
    /// a permanent 429.
    #[test]
    fn the_budget_admits_a_maximum_size_body() {
        assert!(TRANSACTION_BODY_BYTES_IN_FLIGHT >= transaction_body_limit::<MainnetV0>());
    }

    /// A single body is charged against the budget and released once the request completes, so
    /// sequential requests do not accumulate. Without the release this would 429 on the second.
    #[tokio::test]
    async fn budget_is_released_when_a_request_completes() {
        // A budget with room for exactly one maximum-size body.
        let limit = transaction_body_limit::<MainnetV0>();
        let budget = Arc::new(Semaphore::new(limit));
        let app = budget_test_app(budget, limit);

        for attempt in 0..3 {
            let res = app.clone().oneshot(sized_post(limit)).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK, "attempt {attempt} was rejected");
        }
    }

    /// The property the budget exists for: concurrent bodies are bounded in aggregate, not just
    /// individually. `DefaultBodyLimit` caps each one; only this caps their sum.
    #[tokio::test]
    async fn concurrent_bodies_are_bounded_in_aggregate() {
        // Room for two of the three bodies below.
        let body_len = 1024 * 1024;
        let budget = Arc::new(Semaphore::new(body_len * 2));
        let app = budget_test_app(budget.clone(), transaction_body_limit::<MainnetV0>());

        // Hold two bodies' worth, standing in for two requests already in flight.
        let held = budget.clone().try_acquire_many_owned((body_len * 2) as u32).unwrap();
        let res = app.clone().oneshot(sized_post(body_len)).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS, "a third body was admitted over the budget");

        // Once they finish, the next request is admitted again.
        drop(held);
        let res = app.oneshot(sized_post(body_len)).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    /// A chunked body declares no length, so it is charged the full per-request limit rather than
    /// nothing - otherwise the budget could be bypassed by omitting one header.
    #[tokio::test]
    async fn a_body_with_no_content_length_is_charged_the_full_limit() {
        let limit = transaction_body_limit::<MainnetV0>();
        // Room for the limit, less one byte: enough for any honest small body, but not for a
        // request charged the maximum.
        let budget = Arc::new(Semaphore::new(limit - 1));
        let app = budget_test_app(budget, limit);

        let request =
            Request::builder().method("POST").uri("/transaction/broadcast").body(Body::from(vec![0u8; 16])).unwrap();
        // A hand-built request carries no `Content-Length` unless one is set, which is the case
        // this exercises - the assertion keeps it that way if `Body::from` ever starts adding one.
        assert!(request.headers().get(axum::http::header::CONTENT_LENGTH).is_none());

        let res = app.oneshot(request).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS, "an unmeasured body was charged less than the limit");
    }

    /// The transaction body limit is only useful if it actually reaches the handler: it is applied
    /// to the route rather than the router, so it has to override the blanket `DEFAULT_BODY_LIMIT`
    /// layer rather than be overridden by it. That is a property of axum's layer ordering, not of
    /// our own code, which is why it is pinned rather than assumed - `DefaultBodyLimit` only
    /// inserts a request extension, and `Route::layer` wraps each successive layer on the outside,
    /// so the route-level one inserts last and wins.
    ///
    /// This builds a replica of the production layering rather than exercising `routes()` itself,
    /// which needs a full `Rest` and therefore a ledger. So it pins axum's behavior, not our use
    /// of it: removing the `.layer` from the real `/transaction/broadcast` route would not fail
    /// this test.
    #[tokio::test]
    async fn transaction_route_limit_overrides_the_default_body_limit() {
        async fn echo_len(body: axum::body::Bytes) -> String {
            body.len().to_string()
        }

        // Mirror the production layering: a route-specific limit inside, the blanket limit outside.
        let app = Router::new()
            .route(
                "/transaction/broadcast",
                post(echo_len).layer(DefaultBodyLimit::max(transaction_body_limit::<MainnetV0>())),
            )
            .route("/other", post(echo_len))
            .layer(DefaultBodyLimit::max(DEFAULT_BODY_LIMIT));

        // A body over the blanket limit but under the transaction limit: accepted on the
        // transaction route, rejected on any other.
        let len = DEFAULT_BODY_LIMIT + 1;
        assert!(len < transaction_body_limit::<MainnetV0>(), "the two limits must differ for this to test anything");

        let request = |uri: &str| Request::builder().method("POST").uri(uri).body(Body::from(vec![0u8; len])).unwrap();

        let res = app.clone().oneshot(request("/transaction/broadcast")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK, "the transaction route rejected a body within its own limit");

        let res = app.oneshot(request("/other")).await.unwrap();
        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE, "the route limit leaked onto an unrelated route");
    }

    fn test_app() -> Router {
        let build_routes = || {
            Router::new()
                .route("/not_found", get(|| async { Err::<(), RestError>(RestError::not_found(anyhow!("missing"))) }))
                .route("/bad_request", get(|| async { Err::<(), RestError>(RestError::bad_request(anyhow!("bad"))) }))
                .route(
                    "/service_unavailable",
                    get(|| async { Err::<(), RestError>(RestError::service_unavailable(anyhow!("gone"))) }),
                )
        };
        let router_v1 = build_routes().route_layer(middleware::map_response(v1_error_middleware));
        let router_v2 = Router::new().nest(&format!("/{API_VERSION_V2}"), build_routes());
        router_v1.merge(router_v2)
    }

    #[tokio::test]
    async fn v1_routes_force_internal_server_error() {
        let app = test_app();

        let res = app.clone().oneshot(Request::builder().uri("/not_found").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let res =
            app.clone().oneshot(Request::builder().uri("/bad_request").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let res =
            app.oneshot(Request::builder().uri("/service_unavailable").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn v2_routes_return_specific_errors() {
        let app = test_app();

        let res =
            app.clone().oneshot(Request::builder().uri("/v2/not_found").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        let res =
            app.clone().oneshot(Request::builder().uri("/v2/bad_request").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let res =
            app.oneshot(Request::builder().uri("/v2/service_unavailable").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
