#!/bin/bash

######################################
# Utility functions for devnet scripts
######################################

# Array to store PIDs of all processes
declare -a PIDS

# Flag is set true once a node process stopped
node_stopped=false

# Handler for a child process exiting
function child_exit_handler() {
  # only set to true if this was indeed a node
  for i in "${!PIDS[@]}"; do
    if [[ "${PIDS[i]}" -eq "$pid" ]]; then
      echo "Node #${i} (pid=$pid) exited"
      node_stopped=true
    fi
  done
}

# Function checking that each node reached a sufficient block height.
function check_heights() {
  echo "Checking block heights on all nodes..."

  local total_validators=$1
  local total_clients=$2
  local min_height=$3
  local network_name=$4

  local all_reached=true
  local highest_height=0

  for ((node_index = 0; node_index < total_validators + total_clients; node_index++)); do
    port=$((3030 + node_index))
    height=$(curl -s "http://127.0.0.1:$port/v2/$network_name/block/height/latest" || echo "0")
    echo "Node $node_index block height: $height"
    
    # Track highest height for reporting
    if [[ "$height" =~ ^[0-9]+$ ]] && (( height > highest_height )); then
      highest_height=$height
    fi
    
    if ! [[ "$height" =~ ^[0-9]+$ ]] || (( height < min_height )); then
      all_reached=false
    fi
  done
  
  if $all_reached; then
    echo "✅ SUCCESS: All nodes reached minimum height of $min_height"
    return 0
  else
    echo "⏳ WAITING: Not all nodes reached minimum height of $min_height (highest so far: $highest_height)"
    return 1
  fi
}

# Function checking that nodes created logs on disk.
function check_logs() {
  echo "Checking logs exist for all nodes..."
  local log_dir=$1
  local total_validators=$2
  local total_clients=$3
  
  local all_reached=true
  local highest_height=0

  for ((validator_index = 0; validator_index < total_validators; validator_index++)); do
    if [ ! -s "$log_dir/validator-${validator_index}.log" ]; then
      echo "❌ Test failed! Validator #${validator_index} did not create any logs in \"$log_dir\"."
      return 1
    fi
  done
  for ((client_index = 0; client_index < total_clients; client_index++)); do
    if [ ! -s "$log_dir/client-${client_index}.log" ]; then
      echo "❌ Test failed! Client #${client_index} did not create any logs in \"$log_dir\"."
      return 1
    fi
  done

  return 0
}

# Determine network name based on network_id
function get_network_name() {
  local network_id=$1

  case $network_id in
    0)
      echo "mainnet"
      ;;
    1)
      echo "testnet"
      ;;
    2)
      echo "canary"
      ;;
    *)
      >&2 echo "Unknown network ID: $network_id, defaulting to mainnet"
      echo "mainnet"
      ;;
  esac
}

# Stops all running processe in the given list.
function stop_nodes() {
  echo "🚨 Cleaning up ${#PIDS[@]} process(es)…"
  for pid in "${PIDS[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then
      kill -9 "$pid" 2>/dev/null || true
    fi
  done
}

# succeeds if all nodes are available.
function check_nodes() {
  local total_validators=$1
  local total_clients=$2

  for ((node_index = 0; node_index < total_validators + total_clients; node_index++)); do
    port=$((3030 + node_index))
    status=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3030/v2/$network_name/version")
    # Fail if the HTTP response is not 2XX.
    if (( status < 200 || status > 300 )); then
      return 1
    fi
  done

  return 0
}

# Blocks until the network is ready.
function wait_for_nodes() {
  echo "Waiting for nodes to become ready"
  
  local total_validators=$1
  local total_clients=$2

  while true; do
    if [ "$node_stopped" = true ]; then
      echo "Something went wrong: one more nodes stopped unexpectedly"
      return 1
    fi
    
    if check_nodes "$total_validators" "$total_clients"; then
      echo "All nodes are ready!"
      return 0
    fi
  done
}
