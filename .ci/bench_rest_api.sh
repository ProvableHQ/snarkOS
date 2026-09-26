#!/bin/bash

###########################################################
# Measures the performance of a node's REST API
#
# Usage:
#   ./.ci/bench_rest_api.sh            Benchmarks the block routes of a local devnet node.
#   ./.ci/bench_rest_api.sh --history  Benchmarks the history routes of a node with an indexed history.
#
# With `--history`:
#   - HISTORY_LEDGER: a ledger directory whose history is backfilled. The script starts a client with
#     `--history` on it. Unset, the script benchmarks the node already serving REST on port 3030,
#     which must run with `--history` and a `--rest-rps` high enough not to rate-limit the benchmark.
#   - HISTORY_PROGRAMS: the `--history-programs` list the history was backfilled with (default: empty,
#     so only staking rewards). The mapping benchmarks query `credits.aleo` and run only if it is listed.
#   - HISTORY_NETWORK_ID: the network of that ledger (default: 0, mainnet).
#   - HISTORY_STARTUP_TIMEOUT: seconds to wait for a started client's REST server (default: 3600).
#     A started client first indexes any blocks the ledger gained since its backfill.
#   - HISTORY_REQUESTS_PER_WORKER: requests each worker sends per benchmark (default: 1000).
###########################################################

set -eo pipefail # error on any command failure

network_id=1

# The size of the validator set.
num_validators=40

# Adjust this to show more/less log messages
log_filter="info,snarkos_node_sync=debug,snarkos_node_tcp=warn,snarkos_node_rest=warn"

#shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh

# Create log directory
init_log_dir

branch_name=$(git rev-parse --abbrev-ref HEAD)
log "On branch: ${branch_name}"

# Define a trap handler that cleans up all processes on exit.
trap stop_nodes EXIT

# Define a trap handler that prints a message when an error occurs.
trap 'log "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Benchmarks the history routes of a node whose history is indexed. See the usage above.
function bench_history() {
  local history_network_id=${HISTORY_NETWORK_ID:-0}
  local history_network_name
  history_network_name=$(get_network_name "$history_network_id")
  log "Using network: $history_network_name (ID: $history_network_id)"

  local history_programs=${HISTORY_PROGRAMS:-}
  if [[ -n "${HISTORY_LEDGER:-}" ]]; then
    log "Starting a history client on the ledger at $HISTORY_LEDGER"
    local history_flags=(
      --nodisplay --nobanner --noupdater # reduce clutter in the output
      "--log-filter=$log_filter" # only show the logs we care about
      "--network=$history_network_id"
      --client --history # serve the history index
      --nocdn --trusted-peers-only # keep the ledger at the height it was backfilled to
      --nojwt
      "--ledger-storage=$HISTORY_LEDGER"
      --rest-rps=1000000 # ensure benchmarks don't fail due to rate limiting
    )
    if [[ -n "$history_programs" ]]; then
      history_flags+=("--history-programs=$history_programs")
    fi
    # shellcheck disable=SC2086
    run_with_prefix "client-0" $TASKSET1 snarkos start "${history_flags[@]}" --logfile="$log_dir/client-0.log"
    PIDS[0]=$!

    # Block until the ledger is loaded and the REST server is up.
    wait_for_nodes 0 1 "$history_network_name" "${HISTORY_STARTUP_TIMEOUT:-3600}"
  else
    log "HISTORY_LEDGER is unset, so benchmarking the node already serving REST on port 3030"
    if ! check_nodes 0 1 "$history_network_name"; then
      log "❌ No node serves REST on port 3030"
      exit 1
    fi
  fi

  local requests_per_worker=${HISTORY_REQUESTS_PER_WORKER:-1000}
  export REST_API_BASE="http://$localhost:3030/v2/$history_network_name"
  if [[ ",$history_programs," == *",credits.aleo,"* ]]; then
    python ./.ci/rest_api_helper.py "history-mapping" "$CORES_PER_NODE" "$requests_per_worker"
    python ./.ci/rest_api_helper.py "history-mapping-latest" "$CORES_PER_NODE" "$requests_per_worker"
    python ./.ci/rest_api_helper.py "history-mapping-batch" "$CORES_PER_NODE" "$requests_per_worker"
  else
    log "Skipping the mapping benchmarks: credits.aleo is not in HISTORY_PROGRAMS"
  fi
  python ./.ci/rest_api_helper.py "history-staking-reward" "$CORES_PER_NODE" "$requests_per_worker"

  log "🎉 History REST API benchmark done!"
}

case "${1:-}" in
  "")
    ;;
  --history)
    bench_history
    exit 0
    ;;
  *)
    log "❌ Unknown argument: $1"
    exit 1
    ;;
esac

network_name=$(get_network_name $network_id)
log "Using network: $network_name (ID: $network_id)"

snapshot_info=$(<info.txt)
log "Snapshot_info: ${snapshot_info}"

# Shared flags between all nodes
common_flags=(
  --nodisplay --nobanner --noupdater # reduce clutter in the output
  "--log-filter=$log_filter" # only show the logs we care about
  "--network=$network_id"
  --nocdn # don't sync from CDN, so we only benchmark p2p sync
  "--dev-num-validators=$num_validators"
  "--no-dev-txs" # disable developemnt transaction generation
  --rest-rps=1000000 # ensure benchmarks don't fail due to rate limiting
)

# The node that has the ledger (runs on the first two cores)
# shellcheck disable=SC2086
run_with_prefix "client-0" $TASKSET1 snarkos start --dev 0 --client "${common_flags[@]}" --logfile="$log_dir/client-0.log"
PIDS[0]=$!

# Block until node is running.
wait_for_nodes 0 1 "$network_name"

python ./.ci/rest_api_helper.py "get-block" "$CORES_PER_NODE" 60
python ./.ci/rest_api_helper.py "block-height" "$CORES_PER_NODE" 10000
python ./.ci/rest_api_helper.py "get-latest-block" "$CORES_PER_NODE" 100

log "🎉 Rest API benchmark done!"
exit 0
