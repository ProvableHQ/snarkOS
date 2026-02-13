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

# Default values if not provided.
: "${total_validators:=7}"
: "${network_id:=0}"
: "${restart_interval:=10}"
: "${final_height:=100}"
: "${num_restarts:=3}"

# The time that is used to determine the total timeout for the test.
max_wait_per_block=10

network_name=$(get_network_name "$network_id")

# Keep verbosity low as we are running many nodes.
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

  run_with_prefix "validator-$validator_index" snarkos start "${common_flags[@]}" "--dev=$validator_index" --validator --logfile="$log_dir/validator-$validator_index.log"
  PIDS[validator_index]=$!
  log "Started validator $validator_index with PID ${PIDS[$validator_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

wait_for_nodes "$total_validators" 0 "$network_name" 180

# Wait longer if there are more blocks to reach.
max_wait=$((final_height * max_wait_per_block))

for iter in $(seq 1 "$num_restarts"); do
  restart_height=$(( iter * restart_interval ))

  # Wait until all nodes reach the restart height.
  if ! wait_for_heights 0 "$total_validators" "$restart_height" "$network_name" $((restart_interval * max_wait_per_block)); then
    log "❌ Test failed! Not all nodes reached restart height of $restart_height within $((restart_interval * max_wait_per_block)) seconds."
    exit 1
  fi
  log "All nodes reached restart height $restart_height. Restarting all validators (iteration $iter/$num_restarts)..."

  # Gracefully shut down all validators
  stop_nodes

  # Wait briefly before restarting
  sleep 5

  # Restart all validators without cleaning their ledger
  for validator_index in $(seq 0 $((total_validators-1))); do
    run_with_prefix "validator-$validator_index" snarkos start "${common_flags[@]}" "--dev=$validator_index" --validator --logfile="$log_dir/validator-$validator_index.log"
    PIDS[validator_index]=$!
    log "Restarted validator $validator_index with PID ${PIDS[$validator_index]}"
    # Add 1-second delay between starting nodes to avoid hitting rate limits
    sleep 1
  done
done

# Wait for final height
if wait_for_heights 0 "$total_validators" "$final_height" "$network_name"; then
 log "SUCCESS!"
  exit 0
else
  log "❌ Test failed! Not all nodes reached final height of $final_height within $max_wait seconds."
  exit 1
fi