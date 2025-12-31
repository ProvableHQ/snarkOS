#!/bin/bash

####################################################
# Backwards-compatibility upgrade test:
#
# 1. Download latest snarkOS release source from GitHub.
# 2. Build it via `cargo install --locked --path . --features test_network`
#    into a separate prefix (SNARKOS_RELEASE_DIR).
# 3. Probe latest_consensus_version for release and PR binaries.
# 4. Compute CONSENSUS_VERSION_HEIGHTS for release and PR.
# 5. Rebuild release snarkOS with its heights.
# 6. Rebuild PR snarkOS with its heights.
# 7. Start a devnet with the release binary.
# 8. Restart nodes one-by-one to the PR binary.
####################################################

set -eo pipefail  # error on any command failure

# --- Parameters from CLI ---
total_validators=$1
total_clients=$2
network_id=$3

# Default values if not provided
: "${total_validators:=4}"
: "${total_clients:=2}"
: "${network_id:=0}"

# Node verbosity
NODE_VERBOSITY=1

# Expected highest consensus version (may be overridden after probing)
EXPECTED_MAX_CONSENSUS_VERSION="${EXPECTED_MAX_CONSENSUS_VERSION:-}"

# How long to wait between upgrades (seconds); used for block-height window
WAIT_BETWEEN_UPGRADES="${WAIT_BETWEEN_UPGRADES:-60}"

# Load shared helpers (is_integer, get_network_name, wait_for_nodes, stop_nodes, ...)
# shellcheck source=utils.sh
. ./.ci/utils.sh

########################################
# Logging helpers
########################################

# Log a message to the console.
function log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

# Checks that the given command is available in the PATH.
function require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "ERROR: required command '$1' not found in PATH" >&2
    exit 1
  fi
}

# Reuse the same target dir for all builds (release + PR) to get incremental builds.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/.ci/target}"
log "Using CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"

##################################################
# Download + build latest stable snarkOS release
##################################################

# Release binary will be installed here via `cargo install --root`.
SNARKOS_RELEASE_DIR="${SNARKOS_RELEASE_DIR:-$PWD/.ci/release-snarkos}"
SNARKOS_RELEASE_BIN="${SNARKOS_RELEASE_BIN:-$SNARKOS_RELEASE_DIR/bin/snarkos}"
SNARKOS_RELEASE_VERSION_FILE="${SNARKOS_RELEASE_VERSION_FILE:-$SNARKOS_RELEASE_DIR/VERSION}"

download_and_build_latest_snarkos() {
  require_cmd curl
  require_cmd tar
  require_cmd cargo

  local force_build="${1:-0}"

  mkdir -p "${SNARKOS_RELEASE_DIR}"

  local repo="ProvableHQ/snarkOS"
  local latest_url latest_tag tar_url
  local tmpdir srcdir
  local existing_tag=""

  log "Resolving latest snarkOS release tag via redirect…"
  latest_url="$(
    curl -fsSL -o /dev/null -w '%{url_effective}' \
      "https://github.com/${repo}/releases/latest"
  )" || {
    echo "ERROR: Failed to resolve latest release URL from GitHub." >&2
    exit 1
  }

  latest_tag="${latest_url##*/}"

  if [ -z "${latest_tag}" ] || [ "${latest_tag}" = "latest" ]; then
    echo "ERROR: Failed to determine latest tag from URL: ${latest_url}" >&2
    exit 1
  fi

  log "Latest stable tag resolved to: ${latest_tag}"

  ########################################
  # Cached binary exists & matches version
  #         AND force_build = 0 → return early
  ########################################
  if [ "$force_build" != "1" ] &&
     [ -x "${SNARKOS_RELEASE_BIN}" ] &&
     [ -f "${SNARKOS_RELEASE_VERSION_FILE}" ]; then

    existing_tag="$(cat "${SNARKOS_RELEASE_VERSION_FILE}" || true)"

    if [ "${existing_tag}" = "${latest_tag}" ]; then
      log "Reusing cached release snarkos for tag ${existing_tag} (no rebuild)."
      return 0
    fi
  fi

  ########################################
  # Cached source exists AND force_build=1
  # → Do NOT download again, just rebuild
  ########################################
  local cached_src_dir=".ci/release-snarkos-src"
  if [ "$force_build" = "1" ] && [ -d "$cached_src_dir" ]; then
    log "Force-rebuilding release snarkos using cached source in $cached_src_dir"
    srcdir="$cached_src_dir"

    (
      cd "$srcdir"
      ci_cargo_install_snarkos --root "${SNARKOS_RELEASE_DIR}"
    ) || {
      echo "ERROR: forced rebuild failed" >&2
      exit 1
    }

    echo "${latest_tag}" > "${SNARKOS_RELEASE_VERSION_FILE}"
    log "Rebuild complete."
    return 0
  fi

  ########################################
  # Need to download the source (first run or version mismatch)
  ########################################

  tar_url="https://github.com/${repo}/archive/refs/tags/${latest_tag}.tar.gz"
  log "Downloading release source tarball: ${tar_url}"

  tmpdir="$(mktemp -d)"
  curl -fL "${tar_url}" -o "${tmpdir}/snarkos-src.tar.gz" || {
    echo "ERROR: Failed to download tarball from ${tar_url}" >&2
    rm -rf "${tmpdir}"
    exit 1
  }

  log "Extracting source tarball…"
  rm -rf "$cached_src_dir"
  mkdir -p "$cached_src_dir"
  tar -xzf "${tmpdir}/snarkos-src.tar.gz" -o -C "$cached_src_dir" --strip-components=1

  srcdir="$cached_src_dir"

  log "Building release snarkos from fresh source at: ${srcdir}"

  (
    cd "${srcdir}"
    ci_cargo_install_snarkos --root "${SNARKOS_RELEASE_DIR}"
  ) || {
    echo "ERROR: cargo install failed for release snarkos" >&2
    rm -rf "${tmpdir}"
    exit 1
  }

  echo "${latest_tag}" > "${SNARKOS_RELEASE_VERSION_FILE}"
  rm -rf "${tmpdir}"

  log "snarkos release (${latest_tag}) built and installed at ${SNARKOS_RELEASE_BIN}"
}

########################################
# Consensus + height helpers
########################################

SNARKOS_CURRENT_BIN="${SNARKOS_CURRENT_BIN:-snarkos}"

network_name=$(get_network_name "$network_id")
log "Using network: $network_name (ID: $network_id)"

log_dir="$PWD/.logs-upgrade-$(date +"%Y%m%d%H%M%S")"
mkdir -p "$log_dir"
chmod 755 "$log_dir"

declare -a PIDS

# Handler that stops all nodes on shutdown.
# shellcheck disable=SC2329
exit_handler() {
  stop_nodes || true
}

# Install signal handlers.
trap exit_handler EXIT
trap 'echo "Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

common_flags=(
  --nodisplay "--network=$network_id" "--verbosity=$NODE_VERBOSITY"
  "--dev-num-validators=$total_validators"
)

# The set of all clients passed to each validators, so that clients can connect to validators.
# NOTE: In newer versions of snarkOS, the set of trusted clients is populated automatically through `--dev-num-clients`
# and this code can be removed eventually.
trusted_peers=$(generate_trusted_clients "$total_validators" "$total_clients")

function start_node() {
  local bin="$1"
  local node_index="$2"
  local role="$3"    # "validator" or "client"
  local log_file="$4"

  local flags=( "${common_flags[@]}" "--dev=$node_index" )

  if [ "$role" = "validator" ]; then
    # The set of other validators to connect to.
    # NOTE: In newever versions of snarkOS, the set of peers is populated automatically through `--dev-num-validators`
    # and this code can be removed evetually. 
    trusted_validators=""
    for ((peer_index = 0; peer_index < total_validators; peer_index++)); do
      if [ "$peer_index" -eq "$node_index" ]; then
        continue
      else
        # append "," if this is not the first trusted validator 
        if [ -n "$trusted_validators" ]; then
          trusted_validators+=","
        fi
        trusted_validators+="127.0.0.1:$((5000+peer_index))"
      fi
    done

    # Validators trust the clients as peers
    flags+=( --validator "--logfile=$log_file" "--peers=$trusted_peers" "--validators=$trusted_validators" )
    if [ "$node_index" -eq 0 ]; then
      flags+=( --metrics --no-dev-txs )
    fi
  else
    flags+=( --client "--logfile=$log_file" )
  fi

  "$bin" start "${flags[@]}" &
  PIDS[node_index]=$!
  log "Started $role $node_index with PID ${PIDS[node_index]} using $(basename "$bin")"
}

stop_node() {
  local node_index="$1"
  local pid="${PIDS[node_index]:-}"

  if [ -z "$pid" ]; then
    return 0
  fi

  if kill -0 "$pid" >/dev/null 2>&1; then
    log "Stopping node index $node_index (PID $pid)…"
    kill "$pid" || true
    local waited=0
    while kill -0 "$pid" >/dev/null 2>&1 && (( waited < 30 )); do
      sleep 1
      waited=$((waited + 1))
    done
    if kill -0 "$pid" >/dev/null 2>&1; then
      log "PID $pid did not exit in time, sending SIGKILL"
      kill -9 "$pid" || true
    fi
  fi
}

get_latest_height() {
  local h
  h=$(curl -s "http://localhost:3030/v2/$network_name/block/height/latest" || echo "")
  if is_integer "$h"; then
    echo "$h"
    return 0
  fi
  return 1
}

get_consensus_and_height() {
  local cv h
  cv=$(curl -s "http://localhost:3030/v2/$network_name/consensus_version" || echo "")
  h=$(curl -s "http://localhost:3030/v2/$network_name/block/height/latest" || echo "")
  echo "$cv $h"
}

# Start a temporary client with $bin just to fetch /version JSON.
fetch_version_json_from_bin() {
  local bin="$1"
  local label="$2"   # for logs
  local version_log="$log_dir/version-${label}.log"

  require_cmd jq
  require_cmd curl

  log "Starting temporary ${label} snarkOS node to fetch version info…"
  "$bin" start --validator "--network=$network_id" --dev 0 --nodisplay --logfile="$version_log" &
  local pid=$!

  local json=""
  local attempts=0
  local max_attempts=30

  # Wait for HTTP API to come up
  while (( attempts < max_attempts )); do
    json="$(curl -s "http://localhost:3030/v2/$network_name/version" || true)"
    if [ -n "$json" ] && echo "$json" | jq -e '.version? // empty' >/dev/null 2>&1; then
      log "Fetched version info from ${label} snarkos."
      break
    fi
    sleep 1
    attempts=$((attempts + 1))
  done

  # Stop temp node
  if kill -0 "$pid" >/dev/null 2>&1; then
    kill "$pid" || true
    local waited=0
    while kill -0 "$pid" >/dev/null 2>&1 && (( waited < 10 )); do
      sleep 1
      waited=$((waited + 1))
    done
    kill -9 "$pid" >/dev/null 2>&1 || true
  fi

  if [ -z "$json" ]; then
    log "WARN: Could not obtain version info from ${label} snarkos."
    return 1
  fi

  echo "$json"
  return 0
}

# Build heights string: for L, we want exactly L entries: 0,5,10,...,5*(L-1)
build_consensus_heights() {
  local lcv="$1"
  local step=5

  if ! is_integer "$lcv" || [ "$lcv" -le 0 ]; then
    echo ""
    return 0
  fi

  local heights=""
  local i=0
  while [ "$i" -lt "$lcv" ]; do
    local h=$(( i * step ))
    if [ -z "$heights" ]; then
      heights="$h"
    else
      heights="$heights,$h"
    fi
    i=$((i + 1))
  done

  echo "$heights"
}

# Build heights string for PR when its latest_consensus_version differs from release:
# same as build_consensus_heights, but last step is +100 instead of +5.
build_consensus_heights_with_big_last() {
  local lcv="$1"
  local step=5

  if ! is_integer "$lcv" || [ "$lcv" -le 0 ]; then
    echo ""
    return 0
  fi

  # Degenerate case: only one consensus version → just "0"
  if [ "$lcv" -eq 1 ]; then
    echo "0"
    return 0
  fi

  local heights=""
  local i=0

  # Up to the penultimate entry with step 5
  while [ "$i" -lt $(( lcv - 1 )) ]; do
    local h=$(( i * step ))
    if [ -z "$heights" ]; then
      heights="$h"
    else
      heights="$heights,$h"
    fi
    i=$((i + 1))
  done

  # Previous value is for i = lcv - 2
  local prev=$(( (lcv - 2) * step ))
  local last=$(( prev + 100 ))
  heights="$heights,$last"

  echo "$heights"
}

ci_cargo_install_snarkos() {
  CARGO_PROFILE_RELEASE_LTO=off \
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
  CARGO_PROFILE_RELEASE_OPT_LEVEL=2 \
  CARGO_PROFILE_RELEASE_DEBUG=0 \
    cargo install --locked --path . --features test_network "$@"
}

# Probe release + PR binaries, compute heights, and rebuild both with those heights.
function derive_consensus_env_from_version() {
  require_cmd jq

  local json_release=""
  local json_current=""
  local lcv_release=""
  local lcv_current=""

  # 1) Release binary: try to get latest_consensus_version; default to 12 if missing.
  if json_release="$(fetch_version_json_from_bin "$SNARKOS_RELEASE_BIN" "release")"; then
    log "Release version JSON: $json_release"
    lcv_release="$(printf '%s\n' "$json_release" \
      | jq -r '.latest_consensus_version // empty' 2>/dev/null || true)"
    if ! is_integer "$lcv_release"; then
      log "Release JSON has no valid latest_consensus_version, defaulting to 12"
      lcv_release="12"
    fi
  else
    log "WARN: Could not fetch version JSON from release binary, defaulting latest_consensus_version to 12"
    lcv_release="12"
  fi

  # 2) PR binary: try to get latest_consensus_version; fall back to release value.
  if json_current="$(fetch_version_json_from_bin "$SNARKOS_CURRENT_BIN" "current")"; then
    log "Current (PR) version JSON: $json_current"
    lcv_current="$(printf '%s\n' "$json_current" \
      | jq -r '.latest_consensus_version // empty' 2>/dev/null || true)"
    if ! is_integer "$lcv_current"; then
      log "Current JSON has no valid latest_consensus_version, falling back to release value $lcv_release"
      lcv_current="$lcv_release"
    fi
  else
    log "WARN: Could not fetch version JSON from current binary, falling back to release value $lcv_release"
    lcv_current="$lcv_release"
  fi

  log "Computed latest_consensus_version (release) = $lcv_release"
  log "Computed latest_consensus_version (current) = $lcv_current"

  # Simplification: step is always 5, and |lcv_current - lcv_release| <= 1.
  local heights_release
  local heights_current

  heights_release="$(build_consensus_heights "$lcv_release")"

  if [ "$lcv_current" = "$lcv_release" ]; then
    # Same consensus horizon → same step-5 pattern
    heights_current="$(build_consensus_heights "$lcv_current")"
  else
    # Different latest consensus version → stretch the last step by +100
    heights_current="$(build_consensus_heights_with_big_last "$lcv_current")"
  fi

  log "Release CONSENSUS_VERSION_HEIGHTS=${heights_release}"
  log "Current CONSENSUS_VERSION_HEIGHTS=${heights_current}"

  # 3) Rebuild release snarkos with its heights.
  log "Rebuilding release snarkos with CONSENSUS_VERSION_HEIGHTS (may re-download source)…"
  FORCE_REBUILD_RELEASE=1 \
  CONSENSUS_VERSION_HEIGHTS="$heights_release" \
  CARGO_PROFILE_RELEASE_LTO=off \
    download_and_build_latest_snarkos 1

  # 4) Rebuild PR snarkos with its heights.
  log "Rebuilding PR snarkos with CONSENSUS_VERSION_HEIGHTS=${heights_current}"
  CARGO_PROFILE_RELEASE_LTO=off \
  CONSENSUS_VERSION_HEIGHTS="$heights_current" \
    ci_cargo_install_snarkos

  # 5) Use PR latest version as EXPECTED_MAX_CONSENSUS_VERSION for the test.
  export CONSENSUS_VERSION_HEIGHTS="$heights_current"
  export EXPECTED_MAX_CONSENSUS_VERSION="$lcv_current"

  log "Derived EXPECTED_MAX_CONSENSUS_VERSION=${EXPECTED_MAX_CONSENSUS_VERSION}"
  log "Exported CONSENSUS_VERSION_HEIGHTS=${CONSENSUS_VERSION_HEIGHTS}"
}

function wait_for_highest_consensus_version() {
  local timeout="${1:-900}"   # default 15 min
  local interval="${2:-10}"
  local elapsed=0

  if [ -n "$EXPECTED_MAX_CONSENSUS_VERSION" ]; then
    log "Waiting for consensus_version >= ${EXPECTED_MAX_CONSENSUS_VERSION}…"
    local last_height=""
    while (( elapsed < timeout )); do
      read -r cv h <<< "$(get_consensus_and_height)"
      if is_integer "$cv" && is_integer "$h"; then
        log "consensus_version=$cv height=$h"
        if (( cv >= EXPECTED_MAX_CONSENSUS_VERSION )); then
          if [ -n "$last_height" ] && (( h > last_height )); then
            log "✅ Highest consensus version ${cv} reached and chain progressing."
            return 0
          fi
        fi
        last_height="$h"
      else
        log "WARN: invalid consensus or height: cv='$cv' h='$h'"
      fi
      sleep "$interval"
      elapsed=$((elapsed + interval))
    done
    echo "❌ Timed out waiting for consensus_version >= ${EXPECTED_MAX_CONSENSUS_VERSION}" >&2
    return 1
  else
    log "EXPECTED_MAX_CONSENSUS_VERSION not set – waiting for stable consensus_version…"
    local last_cv=""
    local last_h=""
    while (( elapsed < timeout )) ; do
      read -r cv h <<< "$(get_consensus_and_height)"
      if is_integer "$cv" && is_integer "$h"; then
        log "consensus_version=$cv height=$h"
        if [ -z "$last_cv" ]; then
          last_cv="$cv"; last_h="$h"
        else
          if (( cv == last_cv )) && (( h >= last_h + 10 )); then
            log "✅ Consensus version $cv appears stable with height from $last_h to $h."
            return 0
          fi
          if (( cv != last_cv )); then
            log "ℹ️ Consensus version changed from $last_cv to $cv at height $h"
          fi
          last_cv="$cv"
          last_h="$h"
        fi
      else
        log "WARN: invalid consensus or height: cv='$cv' h='$h'"
      fi
      sleep "$interval"
      elapsed=$((elapsed + interval))
    done
    echo "❌ Timed out waiting for stable consensus version" >&2
    return 1
  fi
}

function wait_for_height_increase_window() {
  local previous_height="$1"
  local duration="${2:-60}"
  local interval="${3:-5}"

  local elapsed=0
  local increased=0
  local current_height="$previous_height"

  log "Waiting ${duration}s window to see height increase above $previous_height…"

  while (( elapsed < duration )); do
    if current_height="$(get_latest_height)"; then
      log "Current height=${current_height}"
      if (( current_height > previous_height )); then
        increased=1
      fi
    else
      log "WARN: could not fetch latest height"
    fi
    sleep "$interval"
    elapsed=$((elapsed + interval))
  done

  if (( increased == 1 )); then
    log "✅ Height increased from $previous_height to $current_height during upgrade window."
    return 0
  else
    echo "❌ Timeout: height did not increase above $previous_height during ${duration}s." >&2
    return 1
  fi
}

########################################
# MAIN
########################################

log "Starting upgrade_nodes_ci.sh"
log "total_validators=${total_validators}, total_clients=${total_clients}, network_id=${network_id}"
log "EXPECTED_MAX_CONSENSUS_VERSION=${EXPECTED_MAX_CONSENSUS_VERSION:-<none>} WAIT_BETWEEN_UPGRADES=${WAIT_BETWEEN_UPGRADES}"

# 1. Build release snarkos once (without heights override yet).
download_and_build_latest_snarkos 0

# 2–4. Probe versions, compute heights, rebuild release & PR with those heights.
derive_consensus_env_from_version

# From here on, we use:
#   - SNARKOS_RELEASE_BIN  => rebuilt with release heights
#   - snarkos (PR)         => rebuilt with PR heights

log "Cleaning dev stores with release binary..."
for ((node_index = 0; node_index < total_validators + total_clients; node_index++)); do
  "$SNARKOS_RELEASE_BIN" clean "--dev=$node_index" "--network=$network_id"
done

log "Starting $total_validators validator nodes with release binary..."
for ((validator_index = 0; validator_index < total_validators; validator_index++)); do
  log_file="$log_dir/validator-$validator_index.log"
  start_node "$SNARKOS_RELEASE_BIN" "$validator_index" "validator" "$log_file"
  sleep 1
done

log "Starting $total_clients client nodes with release binary..."
for ((client_index = 0; client_index < total_clients; client_index++)); do
  node_index=$((client_index + total_validators))
  log_file="$log_dir/client-$client_index.log"
  start_node "$SNARKOS_RELEASE_BIN" "$node_index" "client" "$log_file"
  if (( client_index < total_clients-1 )); then
    sleep 1
  fi
done

wait_for_nodes "$total_validators" "$total_clients"
wait_for_highest_consensus_version 900 10

total_nodes=$((total_validators + total_clients))

for ((node_index = 0; node_index < total_nodes; node_index++)); do
  if (( node_index < total_validators )); then
    role="validator"
    idx_label="$node_index"
    log_file="$log_dir/validator-$node_index.log"
  else
    role="client"
    idx_label=$((node_index - total_validators))
    log_file="$log_dir/client-$idx_label.log"
  fi

  log "=============================="
  log "Upgrading ${role} ${idx_label} (node index ${node_index})"
  log "=============================="

  baseline_height="$(get_latest_height || echo 0)"

  stop_node "$node_index"
  start_node "$SNARKOS_CURRENT_BIN" "$node_index" "$role" "$log_file"

  if ! wait_for_height_increase_window "$baseline_height" "$WAIT_BETWEEN_UPGRADES" 5; then
    echo "❌ Upgrade failed: chain did not advance after restarting ${role} ${idx_label} (node index ${node_index})."
    echo "Last 50 lines of ${role} ${idx_label} log:"
    tail -n 50 "$log_file" || true
    exit 1
  fi
done

log "Upgrade test passed: network reached highest consensus version with release, all nodes upgraded to PR snarkos, and consensus version remained correct."

exit 0
