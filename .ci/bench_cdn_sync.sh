#!/bin/bash

####################################################
# Measures a client syncing 1000 blocks from the CDN
####################################################

set -eo pipefail # error on any command failure

network_id=0
min_height=250

# Adjust this to show more/less log messages
log_filter="info,snarkos_node_rest=warn,snarkos_node_cdn=debug"

max_wait=1800 # Wait for up to 30 minutes
poll_interval=1 # Check block heights every second

. ./.ci/utils.sh

network_name=$(get_network_name $network_id)
echo "Using network: $network_name (ID: $network_id)"

# Create log directory
log_dir=".logs-$(date +"%Y%m%d%H%M%S")"
mkdir -p "$log_dir"

# Define a trap handler that cleans up all processes on exit.
function exit_handler() {
  stop_nodes
}
trap exit_handler EXIT
trap child_exit_handler CHLD

# Define a trap handler that prints a message when an error occurs.
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Ensure there are no old ledger files and the node syncs from scratch
snarkos clean "--network=$network_id" || true

# Arguments to pass to the node
args=(
  "--network=$network_id"
  --nobanner --noupdater --nodisplay # reduce clutter in the output and hide TUI
  --rest-rps=1000000 # ensure benchmarks don't fail due to rate limiting
  --log-filter=$log_filter # only show the logs we care about
)

# Spawn the client that will sync the ledger.
# Use the same CPU cores as in the other benchmarks, so the numbers are comparable.
$TASKSET2 snarkos start --client "${args[@]}" &
PIDS[0]=$!

wait_for_nodes 0 1

# Check heights periodically with a timeout
SECONDS=0
while (( SECONDS < max_wait )); do
  if check_heights 0 1 $min_height "$network_name" "$SECONDS"; then
    total_wait=$SECONDS
    throughput=$(compute_throughput "$min_height" "$total_wait")

    echo "🎉 Benchmark done! Waited ${total_wait}s for $min_height blocks. Throughput was $throughput blocks/s."

    # Append data to results file.
    printf "{ \"name\": \"cdn-sync\", \"unit\": \"blocks/s\", \"value\": %.3f, \"extra\": \"total_wait=%is, target_height=${min_height}\" }\n" \
       "$throughput" "$total_wait" | tee -a results.json
    exit 0
  fi
  
  # Continue waiting
  sleep $poll_interval
done

echo "❌ Benchmark failed! Client did not sync within 30 minutes."

exit 1
