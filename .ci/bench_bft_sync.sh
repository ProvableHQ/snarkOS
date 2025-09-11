#!/bin/bash

#################################################################
# Measures a validator syncing 1000 blocks from another validator
#################################################################

set -eo pipefail # error on any command failure

network_id=1
min_height=250

# The number of validators that are syncing
num_nodes=1

# Adjust this to show more/less log messages
log_filter="info,snarkos_node_sync=debug,snarkos_node_bft::primary=warn"

max_wait=1800 # Wait for up to 30 minutes
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
function exit_handler() {
  stop_nodes
}
trap exit_handler EXIT
trap child_exit_handler CHLD

# Define a trap handler that prints a message when an error occurs 
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Shared flags betwen all nodes
common_flags=(
  --nobanner --noupdater --nodisplay "--network=$network_id" --nocdn
  --dev-num-validators=40 --no-dev-txs "--log-filter=$log_filter"
)

# The validator that has the ledger to by synced from.
$TASKSET1 snarkos start --dev 0 --validator "${common_flags[@]}" \
  --logfile="$log_dir/validator-0.log" &
PIDS[0]=$!

# Stores the list of all validators.
validators="127.0.0.1:5000"

# Spawn the clients that will sync the ledger
for node_index in $(seq 1 "$num_nodes"); do
  name="validator-$node_index"

  # Ensure there are no old ledger files and the node syncs from scratch
  snarkos clean "--dev=$node_index" "--network=$network_id" || true

  $TASKSET2 snarkos start "--dev=$node_index" --validator \
    "${common_flags[@]}" "--validators=$validators" \
    "--logfile=$log_dir/$name.log" &
  PIDS[node_index]=$!

  # Add the validators BFT address to the validators list.
  bft_port=$((5000 + node_index))
  validators="$validators,127.0.0.1:$bft_port"

  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Block until nodes are running and connected to each other.
wait_for_nodes $((num_nodes+1)) 0

SECONDS=0

# TODO add API call for number of connected validators.
#for ((node_index = 0; node_index < num_nodes+1; node_index++)); do
#  if ! (wait_for_peers "$node_index" $num_nodes); then
#    exit 1
#  fi
#done

connect_time=$SECONDS
echo "ℹ️ Nodes are fully connected (took $connect_time secs). Starting block sync measurement."

# Check heights periodically with a timeout
SECONDS=0
while (( SECONDS < max_wait )); do
  if check_heights 1 $((num_nodes+1)) $min_height "$network_name" "$SECONDS"; then
    total_wait=$SECONDS
    throughput=$(compute_throughput "$min_height" "$total_wait")

    echo "🎉 BFT sync benchmark done! Waited $total_wait seconds for $min_height blocks. Throughput was $throughput blocks/s."

    # Append data to results file.
    printf "{ \"name\": \"bft-sync\", \"unit\": \"blocks/s\", \"value\": %.3f, \"extra\": \"total_wait=%is, target_height=%i, connect_time=%i, branch=%s, %s\" },\n" \
       "$throughput" "$total_wait" "$min_height" "$connect_time" "$branch_name" "$snapshot_info"| tee -a results.json
    exit 0
  fi
  
  # Continue waiting
  sleep $poll_interval
done

echo "❌ Benchmark failed! Validators did not sync within 30 minutes."

# Print logs for debugging
echo "Last 20 lines of validators logs:"
for ((node_index = 0; node_index <= num_nodes; node_index++)); do
  echo "=== Node $node_index logs ==="
  tail -n 20 "$log_dir/validator-$node_index.log"
done

exit 1
