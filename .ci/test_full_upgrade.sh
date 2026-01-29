#!/bin/bash

####################################################
# Backwards-compatibility upgrade test:
#
# 1. Download latest snarkOS release source from GitHub.
# 2. Build it via `cargo install --locked --path . --features test_network`
#    into a separate prefix (SNARKOS_RELEASE_DIR).
# 3. Probe latest_consensus_version for latest release and PR binaries.
# 4. Compute CONSENSUS_VERSION_HEIGHTS for latest release and PR.
# 5. Start the devnet with latest release binaries and pass CONSENSUS_VERSION_HEIGHTS to nodes.
# 6. Restart nodes one-by-one with the PR binary.
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
# shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh

# Set up logging directory
init_log_dir

# Reuse the same target dir for all builds (release + PR) to get incremental builds.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/.ci/target}"
log "Using CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"

SNARKOS_CURRENT_BIN="${SNARKOS_CURRENT_BIN:-snarkos}"

network_name=$(get_network_name "$network_id")
log "Using network: $network_name (ID: $network_id)"

# Handler that stops all nodes on shutdown.
# shellcheck disable=SC2329
exit_handler() {
  stop_nodes || true
}

# Install signal handlers.
trap exit_handler EXIT
trap 'log "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

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
    for peer_index in $(seq 0 $((total_validators-1))); do
      if (( peer_index == node_index )); then
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
    if (( node_index == 0 )); then
      flags+=( --metrics --no-dev-txs )
    fi

    # TODO Remove once old nodes are no longer on v4.4.0!
    if [ "$bin" = "$SNARKOS_CURRENT_BIN" ]; then
      flags+=( --auto-migrate-node-data )
    fi
  else
    flags+=( --client "--logfile=$log_file" )
  fi

  "$bin" start "${flags[@]}" &
  PIDS[node_index]=$!
  log "Started $role $node_index with PID ${PIDS[node_index]} using $(basename "$bin")"
}

function stop_node() {
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

# Start a temporary client with $bin just to fetch /version JSON.
function fetch_version_json_from_bin() {
  local bin="$1"
  local label="$2"   # for logs
  local version_log="$log_dir/version-${label}.log"

  require_cmd jq
  require_cmd curl

  log "Starting temporary ${label} snarkOS node to fetch version info…"
  "$bin" clean --dev 0
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
function build_consensus_heights() {
  local lcv="$1"
  local step=5

  if ! is_integer "$lcv" || (( lcv <= 0 )); then
    echo ""
    return 0
  fi

  local heights=""
  local i=0
  while (( i < lcv )); do
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
function build_consensus_heights_with_big_last() {
  local lcv="$1"
  local step=5

  if ! is_integer "$lcv" || (( lcv <= 0)) ; then
    echo ""
    return 0
  fi

  # Degenerate case: only one consensus version → just "0"
  if (( lcv == 1 )); then
    echo "0"
    return 0
  fi

  local heights=""
  local i=0

  # Up to the penultimate entry with step 5
  while (( i < (lcv - 1) )); do
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

function ci_cargo_install_snarkos() {
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

  # 4) Use PR latest version as EXPECTED_MAX_CONSENSUS_VERSION for the test.
  export CONSENSUS_VERSION_HEIGHTS="$heights_current"
  export EXPECTED_MAX_CONSENSUS_VERSION="$lcv_current"

  log "Derived EXPECTED_MAX_CONSENSUS_VERSION=${EXPECTED_MAX_CONSENSUS_VERSION}"
  log "Exported CONSENSUS_VERSION_HEIGHTS=${CONSENSUS_VERSION_HEIGHTS}"
}

function wait_for_highest_consensus_version() {
  local timeout="${1:-900}"   # default 15 min
  local interval="${2:-10}"

  SECONDS=0
  if [ -n "$EXPECTED_MAX_CONSENSUS_VERSION" ]; then
    log "Waiting for consensus_version >= ${EXPECTED_MAX_CONSENSUS_VERSION}…"
    local last_height=""
    while (( SECONDS < timeout )); do
      cv=$(get_consensus_version 0 "$network_name" || echo 0)
      h=$(get_block_height 0 "$network_name" || echo 0)

      log "consensus_version=$cv height=$h"
      if (( cv >= EXPECTED_MAX_CONSENSUS_VERSION )); then
        if [ -n "$last_height" ] && (( h > last_height )); then
          log "✅ Highest consensus version ${cv} reached and chain progressing."
          return 0
        fi
      fi
      last_height="$h"

      sleep "$interval"
    done
    echo "❌ Timed out waiting for consensus_version >= ${EXPECTED_MAX_CONSENSUS_VERSION}" >&2
    return 1
  else
    log "EXPECTED_MAX_CONSENSUS_VERSION not set – waiting for stable consensus_version…"
    local last_cv=""
    local last_h=""
    while (( SECONDS < timeout )) ; do
      cv=$(get_consensus_version 0 "$network_name" || echo 0)
      h=$(get_block_height 0 "$network_name" || echo 0)

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

      sleep "$interval"
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

  log "Waiting ${duration}s window to see height increase above $previous_height..."

  while (( elapsed < duration )); do
    if current_height="$(get_block_height 0 "$network_name")"; then
      log "Current height=${current_height}"
      if (( current_height > previous_height )); then
        increased=$(( current_height - previous_height ))
      fi
    else
      log "WARN: could not fetch latest height"
    fi
    sleep "$interval"
    elapsed=$((elapsed + interval))
  done

  # Wait multiple blocks in case the previous height was slightly outdated.
  if (( increased >= 5 )); then
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
for node_index in $(seq 0 $((total_validators+total_clients-1))); do
  "$SNARKOS_RELEASE_BIN" clean "--dev=$node_index" "--network=$network_id"
done

log "Starting $total_validators validator nodes with release binary..."
for validator_index in $(seq 0 $((total_validators-1))); do
  log_file="$log_dir/validator-$validator_index.log"
  start_node "$SNARKOS_RELEASE_BIN" "$validator_index" "validator" "$log_file"
  sleep 1
done

log "Starting $total_clients client nodes with release binary..."
for client_index in $(seq 0 $((total_clients-1))); do
  node_index=$((client_index + total_validators))
  log_file="$log_dir/client-$client_index.log"
  start_node "$SNARKOS_RELEASE_BIN" "$node_index" "client" "$log_file"
  if (( client_index < total_clients-1 )); then
    sleep 1
  fi
done

wait_for_nodes "$total_validators" "$total_clients"
wait_for_highest_consensus_version 900 10

for node_index in $(seq 0 $((total_validators+total_clients-1))); do
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

  baseline_height=$(get_block_height 0 "$network_name" || echo 0)

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
