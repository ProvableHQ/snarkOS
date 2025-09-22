#!/bin/bash

###########################################################
# Measures the performance of a node's REST API
###########################################################

set -eo pipefail # error on any command failure

network_id=1

# Adjust this to show more/less log messages
log_filter="info,snarkos_node_sync=debug,snarkos_node_tcp=warn,snarkos_node_rest=warn"

max_wait=2400 # Wait for up to 40 minutes
poll_interval=1 # Check block heights every second

. ./.ci/utils.sh

branch_name=$(git rev-parse --abbrev-ref HEAD)
echo "On branch: ${branch_name}"

network_name=$(get_network_name $network_id)
echo "Using network: $network_name (ID: $network_id)"

snapshot_info=$(<info.txt)
echo "Snapshot_info: ${snapshot_info}"

# Create log directory
log_dir=".logs-$(date +"%Y%m%d%H%M%S")"
mkdir -p "$log_dir"

# Define a trap handler that cleans up all processes on exit.
trap stop_nodes EXIT
trap child_exit_handler CHLD

# Define a trap handler that prints a message when an error occurs.
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Shared flags betwen all nodes
common_flags=(
  --nodisplay --nobanner --noupdater # reduce clutter in the output
  "--log-filter=$log_filter" # only show the logs we care about
  "--network=$network_id"
  --nocdn # don't sync from CDN, so we only benchmark p2p sync
  --dev-num-validators=40 --no-dev-txs
  --rest-rps=1000000 # ensure benchmarks don't fail due to rate limiting
)

# The client that has the ledger
# (runs on the first two cores)
$TASKSET1 snarkos start --dev 0 --client "${common_flags[@]}" \
  --logfile="$log_dir/client-0.log" &
PIDS[0]=$!

# Block until node is running.
wait_for_nodes 0 1

python ./.ci/rest_api_helper.py "get-block" $CORES_PER_NODE 60
python ./.ci/rest_api_helper.py "block-height" $CORES_PER_NODE 10000
python ./.ci/rest_api_helper.py "get-latest-block" $CORES_PER_NODE 100

exit 0
