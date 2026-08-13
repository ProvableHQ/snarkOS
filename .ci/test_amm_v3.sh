#!/bin/bash

####################################################
# Deploys all amm-v3 programs on a development network
# after the latest consensus version is reached.
####################################################

set -eo pipefail # error on any command failure

# Uncomment this to print commands before executing them for easier debugging.
#set -x

# Set parameters directly
total_validators=$1
total_clients=$2
network_id=$3
max_warnings=$4

# The verbosity of snarkos nodes.
NODE_VERBOSITY=4
# Max logfile sizes (bytes) for regression checks.
MAX_VALIDATOR_LOG_SIZE_BYTES=$((8 * 1024 * 1024))
MAX_CLIENT_LOG_SIZE_BYTES=$((2 * 1024 * 1024))

# Default values if not provided
: "${total_validators:=4}"
: "${total_clients:=4}" # need at least 4 clients, so each validator has at least one client connected to it.
: "${network_id:=0}"
: "${max_warnings:=300}"

# shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh

# Determine network name based on network_id
network_name=$(get_network_name "$network_id")
echo "Using network: $network_name (ID: $network_id)"

# Create log directory
init_log_dir

# Define a trap handler that cleans up all processes on exit.
# shellcheck disable=SC2329
function exit_handler() {
  stop_nodes
}
trap exit_handler EXIT

# Define a trap handler that prints a message when an error occurs
trap 'log "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

require_cmd snarkos
require_cmd leo
require_cmd curl
require_cmd jq

amm_v3_root="${AMM_V3_ROOT:-$PWD/../amm-v3}"
if [[ ! -d "$amm_v3_root" ]]; then
  log "❌ amm-v3 repo not found at $amm_v3_root (clone ProvableHQ/amm-v3 next to snarkOS, or set AMM_V3_ROOT)"
  exit 1
fi
log "Using amm-v3 at $amm_v3_root"

# Flags used by all nodes.
common_flags=(
  --nodisplay --nobanner --noupdater "--network=$network_id" "--verbosity=$NODE_VERBOSITY"
  "--dev-num-validators=$total_validators"  "--dev-num-clients=$total_clients"
)

# Start all validator nodes in the background
for validator_index in $(seq 0 $((total_validators-1))); do
  snarkos clean "--dev=$validator_index" "--network=$network_id"

  log_file="$log_dir/validator-$validator_index.log"
  if (( validator_index == 0 )); then
    run_with_prefix "validator-$validator_index" snarkos start "${common_flags[@]}" "--dev=$validator_index" \
      --validator "--logfile=$log_file" "--rest=127.0.0.1:$((3030+validator_index))" \
      --metrics --no-dev-txs
  else
    run_with_prefix "validator-$validator_index" snarkos start "${common_flags[@]}" "--dev=$validator_index" \
      --validator "--logfile=$log_file" "--rest=127.0.0.1:$((3030+validator_index))"
  fi
  PIDS[validator_index]=$!
  log "Started validator $validator_index with PID ${PIDS[$validator_index]}"

  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Start all client nodes in the background.
for client_index in $(seq 0 $((total_clients-1))); do
  # compute the absolute index for this node.
  node_index=$((client_index + total_validators))

  snarkos clean "--dev=$node_index" "--network=$network_id"

  log_file="$log_dir/client-$client_index.log"
  run_with_prefix "client-$client_index" snarkos start "${common_flags[@]}" "--dev=$node_index" \
    --client "--logfile=$log_file" "--rest=127.0.0.1:$((3030+node_index))"
  PIDS[node_index]=$!
  log "Started client $client_index with PID ${PIDS[$node_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  if (( client_index < total_clients-1)); then
    sleep 1
  fi
done

# Ensure all nodes are up and running.
# Wait up to two minutes, as this can take long in CI.
wait_for_nodes "$total_validators" "$total_clients" "$network_name" 180

# Wait for validators to be fully connected.
log "ℹ️ Waiting for validators to be fully connected..."
for validator_index in $(seq 0 $((total_validators-1))); do
  if ! (wait_for_bft_connections "$validator_index" $((total_validators-1)) "$network_name"); then
    exit 1
  fi
done
log "✅ All validators are fully connected"

if (( total_clients > 0 )); then
  log "ℹ️ Waiting for clients to have at least one peer..."
  # Wait for all clients to be connected to another client or a validator.
  for client_index in $(seq 0 $((total_clients-1))); do
    node_index=$((client_index + total_validators))
    if ! (wait_for_peers "$node_index" 1 "$network_name"); then
      exit 1
    fi
  done
  log "✅ All clients have at least one peer"
fi

if ! wait_for_stable_consensus_version 0 "$network_name"; then
  echo "❌ Test failed! Consensus version did not stabilize within 5 minutes."
  exit 1
fi

consensus_version=$(get_consensus_version 0 "$network_name")
height=$(get_block_height 0 "$network_name")
log "✅ Latest consensus version $consensus_version is stable at height $height"

log "● Deploying amm-v3 programs with --dev-key 0..."
AMM_V3_ROOT="$amm_v3_root" NETWORK_ID="$network_id" ENDPOINT="http://127.0.0.1:3030" \
  ./.ci/deploy_amm_v3.sh --dev-key 0

log "🎉 Test passed! All amm-v3 programs deployed after latest consensus version."

# Ensure no errors are generated during the devnet run, as all nodes are
# expected to operate without failures or interruptions.
if check_logs "$log_dir" "$total_validators" "$total_clients" "$max_warnings" "$MAX_VALIDATOR_LOG_SIZE_BYTES" "$MAX_CLIENT_LOG_SIZE_BYTES"; then
  exit 0
else
  exit 1
fi
