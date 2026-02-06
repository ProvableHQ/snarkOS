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
reset_interval=$3
final_height=$4
num_resets=$5

# Default values if not provided
: "${total_validators:=7}"
: "${network_id:=0}"
: "${reset_interval:=10}"
: "${final_height:=20}"
: "${num_resets:=3}"

max_faulty=$(( (total_validators - 1) / 3 ))
# AleoBFT needs at least N-f for a quorum, not 2*f+1.
majority=$((total_validators - max_faulty))
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

for iter in $(seq 1 "$num_resets"); do
  reset_height=$(( iter * reset_interval ));

  # Block until the reset height is reached.
  wait_for_heights 0 "$total_validators" "$reset_height" "$network_name" $((reset_interval * 5))
  log "All nodes reached the next reset height."

  # Gracefully shut down a majority of the validators
  mapfile -t target_indices < <(generate_random_indices "$majority" $(( ${#PIDS[@]} - 1 )))
  stop_some_nodes "${target_indices[@]}"

  for target_index in "${target_indices[@]}"; do
    # Remove the original ledger
    snarkos clean "--network=$network_id" "--dev=$target_index"
  done

  # wait for a non-trivial amount of time
  sleep 30

  for target_index in "${target_indices[@]}"; do
    # Restart
    snarkos start "${common_flags[@]}" "--dev=$target_index" --validator --logfile="$log_dir/validator-$target_index.log" &
    PIDS[target_index]=$!
    log "Restarted a fresh validator $target_index with PID ${PIDS[$target_index]}"
    # Add 1-second delay between starting nodes to avoid hitting rate limits
    sleep 1
  done
done

if wait_for_heights 0 "$total_validators" "$final_height" "$network_name" "$max_wait"; then
  log "SUCCESS!"
  exit 0
else
  log "❌ Test failed! Not all nodes reached final height of $final_height within $max_wait seconds."
  exit 1
fi