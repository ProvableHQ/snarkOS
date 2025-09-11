#!/bin/bash

######################################
# Utility functions for devnet scripts
######################################

# Array to store PIDs of all processes
declare -a PIDS

# Flag is set true once a node process stopped
node_stopped=false

# Tasksets to pin processes to specfic CPUs.
# This is a no-op on MacOS.
if [[ "$(uname)" == "Darwin" ]]; then
  TASKSET1=""
  TASKSET2=""
else
  TASKSET1="taskset -c 0,1"
  TASKSET2="taskset -c 2,3"
fi

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
      all_reached=false
    fi
  done
  
  if $all_reached; then
    echo "✅ SUCCESS: All nodes reached minimum height of $min_height"
    return 0
  else
    if (( elapsed > 0 && ((elapsed % 60) == 0) )); then
      elapsed_mins=$((elapsed / 60))
      echo "⏳ WAITING: Not all nodes reached minimum height of $min_height (highest so far: $highest_height, elapsed: $elapsed_mins minutes))"
    fi

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

  # block until all nodes have shut down
  wait
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

# Compute the throughput for a number of operation over some time.
function compute_throughput {
  local num_ops=$1
  local duration=$2
  local decimal_points=2
  
  # Use floating point division
  result=$(bc <<< "scale=$decimal_points; $num_ops/$duration")

  echo "$result"
}
