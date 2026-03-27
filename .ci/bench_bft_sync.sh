#!/bin/bash

#################################################################
# Measures a validator syncing 1000 blocks from another validator
#################################################################

set -eo pipefail # error on any command failure

network_id=1
min_height=250

# The total number of validators in the beacon committee.
# This must match the number of validators used when generating the snapshot. 
num_validators=40

# The number of validators that are syncing
num_nodes=1

# Adjust this to show more/less log messages
log_filter="info,snarkos_node_sync=trace,snarkos_node_bft::sync=trace,snarkos_node_bft::primary=warn,snarkos_node_rest=warn"

max_wait=1800 # Wait for up to 30 minutes
poll_interval=1 # Check block heights every second

#shellcheck source=SCRIPTDIR/utils.sh
. .ci/utils.sh

# Create log directory
init_log_dir

branch_name=$(git rev-parse --abbrev-ref HEAD)
log "On branch: ${branch_name}"

network_name=$(get_network_name $network_id)
log "Using network: $network_name (ID: $network_id)"

snapshot_info=$(<info.txt)
log "Snapshot_info: ${snapshot_info}"

# Define a trap handler that cleans up all processes on exit.
# shellcheck disable=SC2329
function exit_handler() {
  stop_nodes
}
trap exit_handler EXIT

# Define a trap handler that prints a message when an error occurs 
trap 'log "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Shared flags betwen all nodes
common_flags=(
  --nobanner --noupdater --nodisplay \
  "--network=$network_id"
  --nocdn
  "--dev-num-clients=0"
  "--dev-num-validators=$num_validators"
  --no-dev-txs
  "--log-filter=$log_filter"
)

# The validator that has the ledger to by synced from.
# shellcheck disable=SC2086
run_with_prefix "validator-0" $TASKSET1 snarkos start --dev 0 --validator "${common_flags[@]}" \
  --logfile="$log_dir/validator-0.log"
PIDS[0]=$!

# Stores the list of all validators.
validators="127.0.0.1:5000"

# Spawn the validators that will sync the ledger
for node_index in $(seq 1 "$num_nodes"); do
  name="validator-$node_index"

  # Ensure there are no old ledger files and the node syncs from scratch
  snarkos clean "--dev=$node_index" "--network=$network_id" || true

  # shellcheck disable=SC2086
  run_with_prefix "$name" $TASKSET2 snarkos start "--dev=$node_index" --validator \
    "${common_flags[@]}" "--validators=$validators" \
    "--logfile=$log_dir/$name.log"
  PIDS[node_index]=$!

  # Add the validators BFT address to the validators list.
  bft_port=$((5000 + node_index))
  validators="$validators,127.0.0.1:$bft_port"

  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Block until nodes are running and connected to each other.
wait_for_nodes $((num_nodes+1)) 0 "$network_name"

start=$(now)

# Wait for all validators to be connected to each other via the gateway.
for ((node_index = 0; node_index < num_nodes+1; node_index++)); do
  if ! (wait_for_bft_connections "$node_index" $num_nodes "$network_name"); then
    exit 1
  fi
done

connect_time=$(elapsed_since "$start")
log "ℹ️ Nodes are fully connected (took $connect_time secs). Starting block sync measurement."

# Check heights periodically with a timeout
start=$(now)
while (( $(elapsed_since "$start") < max_wait )); do
  # The last block cannot be fully applied to the ledger yet as there is no next block to confirm it.
  # However, we know that the sync height of a node is always at least one more than the ledger height.
  expected_height=$((min_height-1))
  total_wait=$(elapsed_since "$start")
  
  if check_heights 1 $((num_nodes+1)) $expected_height "$network_name" "$total_wait"; then
    throughput=$(compute_throughput "$min_height" "$total_wait")

    log "🎉 BFT sync benchmark done! Waited $total_wait seconds for $min_height blocks. Throughput was $throughput blocks/s."

    # Append data to results file.
    printf "{ \"name\": \"bft-sync\", \"unit\": \"blocks/s\", \"value\": %.3f, \"extra\": \"total_wait=%is, target_height=%i, connect_time=%i, branch=%s, %s\" },\n" \
       "$throughput" "$total_wait" "$min_height" "$connect_time" "$branch_name" "$snapshot_info"| tee -a results.json
    exit 0
  fi
  
  # Continue waiting
  sleep "$poll_interval"
done

log "❌ Benchmark failed! Validators did not sync within 30 minutes."
print_validator_logs "$log_dir" "$num_validators" "$num_nodes"

exit 1
