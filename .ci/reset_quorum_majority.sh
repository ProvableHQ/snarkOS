#!/bin/bash

#shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh

# Network parameters
total_validators=7
majority=$(( (total_validators - 1) / 3 + 1 ))
network_id=0
network_name="mainnet"

# Stopping conditions
restart_height=10
final_height=20

# Define a trap handler that cleans up all processes on exit.
trap stop_nodes EXIT

# Define a trap handler that prints a message when an error occurs 
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Start all validator nodes in the background
for ((validator_index = 0; validator_index < total_validators; validator_index++)); do
  snarkos clean --dev $validator_index --network=$network_id

  snarkos start --nodisplay --network $network_id --dev $validator_index --dev-num-validators $total_validators --validator &
  PIDS[validator_index]=$!
  echo "Started validator $validator_index with PID ${PIDS[$validator_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

wait_for_nodes "$total_validators" 0 

total_wait=0
while true; do
  if check_heights 0 "$total_validators" "$restart_height" "$network_name"; then
    echo "All nodes reached restart height."

    # Gracefully shut down a majority of the validators
    targets=( $(generate_random_indices "$majority" $(( ${#PIDS[@]} - 1 ))) )
    stop_some_nodes "${targets[@]}"

    for target_index in "${targets[@]}"; do
      # Remove the original ledger
      snarkos clean "--network=$network_id" "--dev=$target_index"
    done

    # wait for a non-trivial amount of time
    sleep 30

    for target_index in "${targets[@]}"; do
      # Restart
      snarkos start --nodisplay "--network=$network_id" "--dev=$target_index" "--dev-num-validators=$total_validators" \
        --validator &
      PIDS[target_index]=$!
      echo "Restarted a fresh validator $target_index with PID ${PIDS[$target_index]}"
      # Add 1-second delay between starting nodes to avoid hitting rate limits
      sleep 1
    done

    total_wait=$((total_wait + 30 + $majority))

    break
  fi

  sleep 3
  total_wait=$((total_wait + 3))
done

while (( total_wait < 600 )); do  # 10 minutes max
  if check_heights 0 "$total_validators" "$final_height" "$network_name"; then
    echo "SUCCESS!"
    exit 0
  fi

  # Continue waiting
  sleep 3
  total_wait=$((total_wait + 3))
  echo "Waited $total_wait seconds so far..."
done

# The main loop has expired by now
echo "❌ Test failed!"
exit 1
