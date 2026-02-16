#!/bin/bash

###########################################################
# Measures the performance of a node's REST API
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

network_name=$(get_network_name $network_id)
log "Using network: $network_name (ID: $network_id)"

snapshot_info=$(<info.txt)
log "Snapshot_info: ${snapshot_info}"

# Define a trap handler that cleans up all processes on exit.
trap stop_nodes EXIT

# Define a trap handler that prints a message when an error occurs.
trap 'log "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

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
