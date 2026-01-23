#!/bin/bash

######################################
# Utility functions for devnet scripts
######################################

# Array to store PIDs of all processes
declare -a PIDS

# The log directory variable. Will be initialized by init_log_dirl
log_dir="$PWD/.logs-$(date +"%Y%m%d%H%M%S")"

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
      log "Node #${i} (pid=$pid) has exited unexpectedly"
      return 0
    fi
  done
  return 1
}

# Set up a logging directory that node and ci-runner logs are stored in
function init_log_dir() {
  mkdir -p "$log_dir"
  chmod 755 "$log_dir"
  log "Created log directory: $log_dir"
}

# Checks that the given command is available in the PATH.
function require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "ERROR: required command '$1' not found in PATH" >&2
    exit 1
  fi
}

# Write a log message to the console and "ci-runner.log".
function log() {
  msg="[$(date '+%Y-%m-%d %H:%M:%S')] $*"
  echo "$msg" >> "$log_dir/ci-runner.log"
  echo "$msg"
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

  if (( end_index <= start_index )); then
    log "❌ Invalid range: $end_index <= $start_index"
    exit 1 
  fi

  for node_index in $(seq "$start_index" $((end_index-1))); do
    port=$((3030 + node_index))
    height=$(curl -s "http://127.0.0.1:$port/v2/$network_name/block/height/latest" || echo "0")
    
    # Track highest height for reporting
    if (is_integer "$height") && (( height > highest_height )); then
      highest_height=$height
    fi
    
    if ! (is_integer "$height") || (( height < min_height )); then
      log "Node #${node_index} (port=$port) only reached height $height, expected at least $min_height"
      all_reached=false
    fi
  done
  
  if $all_reached; then
    log "✅ SUCCESS: All nodes reached minimum height of $min_height"
    return 0
  else
    if (( elapsed > 0 && ((elapsed % 60) == 0) )); then
      elapsed_mins=$((elapsed / 60))
      log "⏳ WAITING: Not all nodes reached minimum height of $min_height (highest so far: $highest_height, elapsed: $elapsed_mins minutes)"
    fi

    return 1
  fi
}

# Function checking that nodes created logs on disk and they contain no errors.
function check_logs() {
  log "Checking logs exist for all nodes..."
  local log_dir=$1
  local total_validators=$2
  local total_clients=$3
  
  local all_reached=true
  local highest_height=0

  for validator_index in $(seq 0 $((total_validators-1))); do 
    if [ ! -s "$log_dir/validator-${validator_index}.log" ]; then
      log "❌ Test failed! Validator #${validator_index} did not create any logs in \"$log_dir\"."
      return 1
    fi

    if grep -q "ERROR" "$log_dir/validator-${validator_index}.log"; then
      log "❌ Test failed! Validator #${validator_index} logs contain errors."
      # Print the errors to the console.
      grep "ERROR" "$log_dir/validator-${validator_index}.log"
      return 1
    fi
  done

  for client_index in $(seq 0 $((total_clients-1))); do
    if [ ! -s "$log_dir/client-${client_index}.log" ]; then
      log "❌ Test failed! Client #${client_index} did not create any logs in \"$log_dir\"."
      return 1
    fi

    if grep -q "ERROR" "$log_dir/client-${client_index}.log"; then
      log "❌ Test failed! Client #${client_index} logs contain errors."
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

# Stops all running processe in the given list.
function stop_nodes() {
  log "🚨 Cleaning up ${#PIDS[@]} process(es)…"
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
  local network_name=$3

  for node_index in $(seq 0 $((total_validators+total_clients-1))); do
    port=$((3030 + node_index))
    status=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:$port/v2/$network_name/version")
    # Fail if the HTTP response is not 2XX.
    if (( status < 200 || status > 300 )); then
      log "Node #${node_index} (port=$port) is not ready yet"
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
  local network_name=$3

  local max_wait=300
  local poll_interval=1
  local port=$((3030+node_index))

  SECONDS=0
  
  while (( SECONDS < max_wait )); do
    result=$(curl -s "http://localhost:$port/v2/$network_name/peers/count")

    if (is_integer "$result"); then
      if (( result < min_peers )); then
        log "Node #${node_index} (port=$port) has $result peers, expected at least $min_peers. Will wait and retry..."
      else 
        return 0
      fi
    else
      log "Failed to get number of peers for node #${node_index} (port=$port). Will retry..."
      return 0
    fi

    # Continue waiting
    sleep $poll_interval
  done

  log "❌ Nodes did not connect within 5 minutes."
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
  log "Waiting for nodes to become ready"
  
  local total_validators=$1
  local total_clients=$2
  local network_name=$3

  local max_wait=60
  local poll_interval=1

  SECONDS=0

  while (( SECONDS < max_wait )); do
    if check_node_stopped; then
      log "ERROR: one or more nodes stopped unexpectedly"
      return 1
    fi
    
    if check_nodes "$total_validators" "$total_clients" "$network_name"; then
      log  "✅ All nodes are ready!"
      return 0
    fi

    # Pause to give the nodes time to start up.
    sleep $poll_interval
  done

  log "❌ Nodes did not become ready within 60 seconds."
  return 1
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
