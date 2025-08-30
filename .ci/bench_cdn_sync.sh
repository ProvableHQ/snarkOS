#!/bin/bash

####################################################
# Measures a client syncing 1000 blocks from the CDN
####################################################

network_id=0 # CDN sync is tested for mainnet
min_height=250

# Adjust this to show more/less log messages
log_filter="info"

max_wait=600 # Wait for up to ten minutes
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

# Define a trap handler that prints a message when an error occurs 
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Ensure there are no old ledger files and the node syncs from scratch
snarkos clean --network $network_id || true

# Spawn the client that will sync the ledger.
# Use the same CPU cores as in the other benchmarks, so the numbers are comparable.
taskset -c 1,2 snarkos start --nodisplay --network $network_id \
  --client  --log-filter=$log_filter &
PIDS[client_index]=$!

wait_for_nodes 0 1

# Check heights periodically with a timeout
total_wait=0
while (( total_wait < max_wait )); do
  if check_heights 0 1 $min_height "$network_name"; then
    # Use floating point division
    throughput=$(bc <<< "scale=2; $min_height/$total_wait")

    echo "🎉 Test passed!. Waited $total_wait for $min_height blocks. Throughput was $throughput blocks/s."

    # Append data to results file.
    printf "{ \"name\": \"cdn-sync\", \"unit\": \"blocks/s\", \"value\": %.3f, \"extra\": \"total_wait=%is, target_height=${min_height}\" }\n" \
       "$throughput" "$total_wait" | tee -a results.json
    exit 0
  fi
  
  # Continue waiting
  sleep $poll_interval
  total_wait=$((total_wait+poll_interval))
  echo "Waited $total_wait seconds so far..."
done

echo "❌ Test failed! Client did not sync within 5 minutes."

exit 1
