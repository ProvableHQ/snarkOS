# Attention - Change the MSRV in Cargo.toml and rust-toolchain as well
FROM cimg/rust:1.88

# Install GCP
RUN curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg \
  && echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | sudo tee -a /etc/apt/sources.list.d/google-cloud-sdk.list \
  && sudo apt-get update \
  && sudo apt-get install google-cloud-cli -y

# Install LLVM and build essentials
RUN sudo apt-get install -y --no-install-recommends build-essential clang llvm-dev llvm lld pkg-config xz-utils make libssl-dev

# Install sccache
RUN export SCCACHE_VERSION="v0.10.0" && export SCCACHE_PKG="sccache-$SCCACHE_VERSION-x86_64-unknown-linux-musl" \
  && wget "https://github.com/mozilla/sccache/releases/download/$SCCACHE_VERSION/$SCCACHE_PKG.tar.gz" \
  && tar -xvf "$SCCACHE_PKG.tar.gz" \
  && sudo mv "$SCCACHE_PKG/sccache" "/bin/sccache" \
  && rm -rf "~/.cargo/registry" \
  && rm -rf $SCCACHE_PKG $SCCACHE_PKG.tar.gz

# Ensure all rust tools are install
RUN rustup default 1.88 \
  && rustup component add clippy rustc cargo rustfmt \
  && cargo install cargo-audit \
  && cargo install cargo-machete \
  && rustup toolchain add nightly
