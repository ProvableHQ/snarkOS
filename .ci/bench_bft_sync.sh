#!/bin/bash

network_id=1
min_height=250

# The number of validators that are syncing
num_nodes=1

# Adjust this to show more/less log messages
log_filter="warn,snarkos_node_sync=debug"

max_wait=1800 # Wait for up to 30 minutes
poll_interval=1 # Check block heights every second

. ./.ci/utils.sh

network_name=$(get_network_name $network_id)
echo "Using network: $network_name (ID: $network_id)"

info.txt > snapshot_info
echo "Snapshot_info:"
echo ${snapshot_info}

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

# Shared flags betwen all nodes
common_flags=" --nodisplay --network $network_id --nocdn --dev-num-validators=40 \
  --no-dev-txs --log-filter=$log_filter"

# The client that has the ledger
taskset -c 0,1 snarkos start --dev 0 --validator ${common_flags} \
  --logfile="$log_dir/validator-0.log" &
PIDS[0]=$!

# Stores the list of all validators.
validators="127.0.0.1:5000"

# Spawn the clients that will sync the ledger
for ((node_index = 1; node_index <= num_nodes; node_index++)); do
  # Ensure there are no old ledger files and the node syncs from scratch
  snarkos clean --dev $node_index --network $network_id || true
  
  taskset -c 2,3 snarkos start --dev $node_index --validator ${common_flags} \
          --logfile "$log_dir/validator-$node_index.log" \
          --validators=$validators &
  PIDS[node_index]=$!

  # Add the validators BFT address to the validators list.
  bft_port=$((5000 + node_index))
  validators="$validators,127.0.0.1:$bft_port"

  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

wait_for_nodes 0 $((num_nodes+1))

# Check heights periodically with a timeout
total_wait=0
while (( total_wait < max_wait )); do
  if check_heights $((num_nodes+1)) 0 $min_height "$network_name"; then
    # Use floating point division
    throughput=$(bc <<< "scale=2; $min_height/$total_wait")

    echo "🎉 Test passed!. Waited $total_wait for $min_height blocks. Throughput was $throughput blocks/s."

    # Append data to results file.
    printf "{ \"name\": \"bft-sync\", \"unit\": \"blocks/s\", \"value\": %.3f, \"extra\": \"total_wait=%is, target_height=${min_height}, ${snapshot_info}\" },\n" \
       "$throughput" "$total_wait" | tee -a results.json
    exit 0
  fi
  
  # Continue waiting
  sleep $poll_interval
  total_wait=$((total_wait+poll_interval))
  echo "Waited $total_wait seconds so far..."
done

echo "❌ Test failed! Validators did not sync within 30 minutes."

# Print logs for debugging
echo "Last 20 lines of validators logs:"
for ((node_index = 0; node_index <= num_nodes; node_index++)); do
  echo "=== Node $node_index logs ==="
  tail -n 20 "$log_dir/validator-$node_index.log"
done

exit 1
