#!/bin/bash

######################################
# Utility functions for devnet scripts
######################################

# Array to store PIDs of all processes
declare -a PIDS

# How many cores should each node use?
# (Should be half of the number of (v)CPUs)
# NOTE: when you update this, update TASKSET1/2 as well.
# shellcheck disable=SC2034
CORES_PER_NODE=8

# Tasksets to pin processes to specific CPUs.
# This is a no-op on MacOS.
if [[ "$(uname)" == "Darwin" ]]; then
  # shellcheck disable=SC2034
  TASKSET1=""
  # shellcheck disable=SC2034
  TASKSET2=""
else
  # shellcheck disable=SC2034
  TASKSET1="taskset -c 0-7"
  # shellcheck disable=SC2034
  TASKSET2="taskset -c 8-15"
fi

# Check if any tracked node process has exited.
# Returns 0 if a node stopped, 1 otherwise.
function check_node_stopped() {
  for i in "${!PIDS[@]}"; do
    local pid="${PIDS[i]}"
    if ! kill -0 "$pid" 2>/dev/null; then
      echo "Node #${i} (pid=$pid) has exited unexpectedly"
      return 0
    fi
  done
  return 1
}

# Function checking that each node in the given range [start_index, end_index)
# reached a minimum block height.
function check_heights() {
  local start_index=$1
  local end_index=$2
  local min_height=$3
  local network_name=$4
  local elapsed=$5

  local all_reached=true
  local highest_height=0

  for node_index in $(seq "$start_index" $((end_index-1))); do
    port=$((3030 + node_index))
    height=$(curl -s "http://127.0.0.1:$port/v2/$network_name/block/height/latest" || echo "0")
    
    # Track highest height for reporting
    if (is_integer "$height") && (( height > highest_height )); then
      highest_height=$height
    fi
    
    if ! (is_integer "$height") || (( height < min_height )); then
      echo "Node #${node_index} (port=$port) only reached height $height, expected at least $min_height"
      all_reached=false
    fi
  done
  
  if $all_reached; then
    echo "✅ SUCCESS: All nodes reached minimum height of $min_height"
    return 0
  else
    if (( elapsed > 0 && ((elapsed % 60) == 0) )); then
      elapsed_mins=$((elapsed / 60))
      echo "⏳ WAITING: Not all nodes reached minimum height of $min_height (highest so far: $highest_height, elapsed: $elapsed_mins minutes)"
    fi

    return 1
  fi
}

# Function checking that nodes created logs on disk and they contain no errors.
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

    if grep -q "ERROR" "$log_dir/validator-${validator_index}.log"; then
      echo "❌ Test failed! Validator #${validator_index} logs contain errors."
      # Print the errors to the console.
      grep "ERROR" "$log_dir/validator-${validator_index}.log"
      return 1
    fi
  done

  for ((client_index = 0; client_index < total_clients; client_index++)); do
    if [ ! -s "$log_dir/client-${client_index}.log" ]; then
      echo "❌ Test failed! Client #${client_index} did not create any logs in \"$log_dir\"."
      return 1
    fi

    if grep -q "ERROR" "$log_dir/client-${client_index}.log"; then
      echo "❌ Test failed! Client #${client_index} logs contain errors."
      # Print the errors to the console.
      grep "ERROR" "$log_dir/client-${client_index}.log"
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

# Stops all running processes in the PIDS list.
function stop_nodes() {
  echo "🚨 Cleaning up ${#PIDS[@]} process(es)…"
  for pid in "${PIDS[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then
      kill -9 "$pid" 2>/dev/null || true
    fi
  done

  # block until all nodes have shut down
  wait
}

# Generates the given number of random indices up to max_index.
function generate_random_indices() {
  local count=$1
  local max_index=$2

  # Check if count is greater than max_index + 1 (impossible request)
  if (( count > max_index + 1 )); then
    echo "Error: Cannot request more unique indices than exist." >&2
    return 1
  fi

  # shuf -i generates a range (0 to max), -n picks N items
  shuf -i 0-"$max_index" -n "$count"
}

# Stops select running processes from the PIDS list.
function stop_some_nodes() {
  local indices=("$@")
  local killed_pids=()

  echo "🚨 Stopping ${#indices[@]} selected node(s)..."

  for i in "${indices[@]}"; do
    # Get the PID from the global PIDS array using the index
    local pid="${PIDS[$i]}"

    # Check if PID exists (is not empty) and is currently running
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      echo "Killing PIDS[$i] -> $pid"
      kill -9 "$pid" 2>/dev/null || true
      # Add to list of PIDs to wait for specifically
      killed_pids+=("$pid")
    else
      echo "Skipping PIDS[$i] (PID: $pid) - Already dead or invalid."
    fi
  done

  # Wait only for the processes we just killed to ensure they are gone
  if [ ${#killed_pids[@]} -gt 0 ]; then
    wait "${killed_pids[@]}" 2>/dev/null || true
  fi
}

# Succeeds if all nodes are available.
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

# Succeeds if the given string is an integer.
function is_integer() {
  if [[ $1 =~ ^[0-9]+$ ]]; then
    return 0
  else
    return 1
  fi
}

# Succeeds if the given string is a float.
function is_float() {  
  if [[ "$1" =~ ^[+-]?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?$ ]]; then
    return 0
  else
    return 1
  fi
}

# Succeeds if the node with the given index has the specified number of peers (or greater)
function wait_for_peers() {
  local node_index=$1
  local min_peers=$2

  local total_wait=0
  local max_wait=300
  local poll_interval=1
  
  while (( total_wait < max_wait )); do
    result=$(curl -s "http://localhost:3030/v2/$network_name/peers/count")

    if (is_integer "$result") && (( result >= min_peers )); then
      return 0
    fi

    # Continue waiting
    sleep $poll_interval
    total_wait=$((total_wait+poll_interval))
  done

  echo "❌ Nodes did not connect within 5 minutes."
  return 1
}

# Blocks until the node with the given index has at least one peer to sync from (or times out).
function wait_for_sync_peers() {
  local node_index=$1

  local max_wait=300 
  for ((total_wait=0; total_wait < max_wait; ++total_wait)); do
    port=$((3030+node_index))
    result=$(curl -s "http://localhost:${port}/v2/$network_name/sync/peers")
    echo "$result"
    num_peers=$(echo "$result" | jq -r '. | length')

    # Height is set to zero without block locators. So wait for until it is greater than 0 for at least one peer.
    for ((idx=0; idx<num_peers; ++idx)); do
      count=$(echo "$result" | jq -r ".[keys[$idx]]")
      if ((count > 0)); then
        return 0
      fi
    done

    # Continue waiting
    sleep 1
  done
  
  return 1
}

# Blocks until the network is ready.
function wait_for_nodes() {
  echo "Waiting for nodes to become ready"
  
  local total_validators=$1
  local total_clients=$2

  while true; do
    if check_node_stopped; then
      echo "ERROR: one or more nodes stopped unexpectedly"
      return 1
    fi
    
    if check_nodes "$total_validators" "$total_clients"; then
      echo "All nodes are ready!"
      return 0
    fi

    # Pause to give the nodes time to start up.
    sleep 1
  done
}

# Compute the throughput for a number of operation over some time.
function compute_throughput {
  local num_ops=$1
  local duration=$2
  local decimal_points=2
  
  # Use floating point division
  result=$(bc <<< "scale=$decimal_points; $num_ops/$duration")

  echo "$result"
}

# Print the last 20 lines of logs for all nodes.
function print_validator_logs() {
  local log_dir=$1
  local total_validators=$2
  local total_clients=$3

  echo "Last 20 lines of node logs:"
  for ((validator_index = 0; validator_index < total_validators; validator_index++)); do
    echo "=== Validator $validator_index logs ==="
    tail -n 20 "$log_dir/validator-$validator_index.log"
  done
}

function print_client_logs() {
  local log_dir=$1
  local total_validators=$2
  local total_clients=$3

  for ((client_index = 0; client_index < total_clients; client_index++)); do
    echo "=== Client $client_index logs ==="
    node_index=$((total_validators + client_index))
    tail -n 20 "$log_dir/client-$client_index.log"
  done
}

# Generate the trusted peers for the validators as they will not allow connections from unknown peers.
function generate_trusted_clients() {
  local total_validators=$1
  local total_clients=$2

  local result=""

  for ((client_index = 0; client_index < total_clients; client_index++)); do
    node_index=$((client_index + total_validators))
    if (( client_index == 0 )); then
      result+="127.0.0.1:$((4130+node_index))"
    else
      result+=",127.0.0.1:$((4130+node_index))"
    fi
  done

  echo "$result"
}
