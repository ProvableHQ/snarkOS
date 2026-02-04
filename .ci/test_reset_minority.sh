#! /usr/bin/env bash

set -eo pipefail # error on any command failure

# Ensure that we run a recent version of bash
if [ "${BASH_VERSINFO[0]}" -lt 5 ]; then
  echo "Error: This script requires bash version 5.0 or higher."
  exit 1
fi

#shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh

# Set up the logging directory
init_log_dir

# Network parameters
total_validators=$1
network_id=$2
restart_interval=$3
final_height=$4
num_restarts=$5

# Default values if not provided
: "${total_validators:=7}"
: "${network_id:=0}"
: "${restart_interval:=10}"
: "${final_height:=20}"
: "${num_restarts:=3}"

minority=$(( (total_validators - 1) / 3 ))
network_name=$(get_network_name "$network_id")
verbosity=0


# Define a trap handler that cleans up all processes on exit.
trap stop_nodes EXIT

# Define a trap handler that prints a message when an error occurs 
trap 'log "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Define flags used by all nodes.
common_flags=(
  --nodisplay --nobanner --noupdater "--network=$network_id" "--verbosity=$verbosity"
  "--dev-num-validators=$total_validators"
)

# Start all validator nodes in the background
for validator_index in $(seq 0 $((total_validators-1))); do
  snarkos clean "--dev=$validator_index" "--network=$network_id"
 
  snarkos start "${common_flags[@]}" "--dev=$validator_index" --validator --logfile="$log_dir/validator-$validator_index.log" & 
  PIDS[validator_index]=$!

  log "Started validator $validator_index with PID ${PIDS[$validator_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

wait_for_nodes "$total_validators" 0 "$network_name"

# Wait longer if there are more blocks to reach.
max_wait=$((final_height * 5 ))

SECONDS=0
for iter in $(seq 1 "$num_restarts"); do
  restart_height=$(( iter * restart_interval ));

  while true; do
    if check_heights 0 "$total_validators" "$restart_height" "$network_name"; then
      log "All nodes reached restart height."

      # Gracefully shut down a minority of the validators
      mapfile -t target_indices < <(generate_random_indices "$minority" $(( ${#PIDS[@]} - 1 )))
      stop_some_nodes "${target_indices[@]}"

      for target_index in "${target_indices[@]}"; do
        # Remove the original ledger
        snarkos clean "--network=$network_id" "--dev=$target_index"
        # Wait until the cleanup concludes
        sleep 1
        # Restart
        snarkos start "${common_flags[@]}" "--dev=$target_index" --validator --logfile="$log_dir/validator-$target_index.log" &
        PIDS[target_index]=$!
        log "Restarted a fresh validator $target_index with PID ${PIDS[$target_index]}"
        # Add 1-second delay between starting nodes to avoid hitting rate limits
        sleep 1
      done

      break
    fi

    sleep 3
  done
done

while (( SECONDS < max_wait )); do  # 10 minutes max
  if check_heights 0 "$total_validators" "$final_height" "$network_name"; then
    log "SUCCESS!"
    exit 0
  fi

  # Continue waiting
  sleep 3
  log "Waited $SECONDS seconds so far..."
done

# The main loop has expired by now
log "❌ Test failed! Not all nodes reached final height of $final_height within $max_wait seconds."
exit 1
