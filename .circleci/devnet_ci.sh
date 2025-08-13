#!/bin/bash
set -eo pipefail # error on any command failure
set -x # print commands before executing them for easier debugging

# Set parameters directly
total_validators=$1
total_clients=$2
network_id=$3
min_height=$4

# Default values if not provided
: "${total_validators:=4}"
: "${total_clients:=2}"
: "${network_id:=0}"
: "${min_height:=60}" # To likely go past the 100 round garbage collection limit.

# Determine network name based on network_id
case $network_id in
  0)
    network_name="mainnet"
    ;;
  1)
    network_name="testnet"
    ;;
  2)
    network_name="canary"
    ;;
  *)
    echo "Unknown network ID: $network_id, defaulting to mainnet"
    network_name="mainnet"
    ;;
esac

echo "Using network: $network_name (ID: $network_id)"

# Create log directory
log_dir=${PWD}".logs-$(date +"%Y%m%d%H%M%S")"
mkdir -p "$log_dir"
chmod 755 "$log_dir"

# Array to store PIDs of all processes
declare -a PIDS

# Define a cleanup function to kill all processes on exit
function cleanup() {
  echo "🚨 Cleaning up ${#PIDS[@]} process(es)…"
  for pid in "${PIDS[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then
      kill -9 "$pid" 2>/dev/null || true
    fi
  done
}
trap cleanup EXIT
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Start all validator nodes in the background
for ((validator_index = 0; validator_index < $total_validators; validator_index++)); do
  snarkos clean --dev $validator_index

  log_file="$log_dir/validator-$validator_index.log"
  if [ "$validator_index" -eq 0 ]; then
    snarkos start --nodisplay --network $network_id --dev $validator_index --allow-external-peers --dev-num-validators $total_validators --validator --logfile $log_file --metrics --no-dev-txs --dev-max-batch-delay-ms 100 &
  else
    snarkos start --nodisplay --network $network_id --dev $validator_index --allow-external-peers --dev-num-validators $total_validators --validator --logfile $log_file --dev-max-batch-delay-ms 100 &
  fi
  PIDS[$validator_index]=$!
  echo "Started validator $validator_index with PID ${PIDS[$validator_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Start all client nodes in the background
for ((client_index = 0; client_index < $total_clients; client_index++)); do
  node_index=$((client_index + total_validators))

  snarkos clean --dev $node_index

  log_file="$log_dir/client-$client_index.log"
  snarkos start --nodisplay --network $network_id --dev $node_index --dev-num-validators $total_validators --client --logfile $log_file &
  PIDS[$node_index]=$!
  echo "Started client $client_index with PID ${PIDS[$node_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  if [ $client_index -lt $((total_clients - 1)) ]; then
    sleep 1
  fi
done

# Function checking that each node reached a sufficient block height.
check_heights() {
  echo "Checking block heights on all nodes..."
  all_reached=true
  highest_height=0
  for ((node_index = 0; node_index < $((total_validators + total_clients)); node_index++)); do
    port=$((3030 + node_index))
    height=$(curl -s "http://127.0.0.1:$port/$network_name/block/height/latest" || echo "0")
    echo "Node $node_index block height: $height"
    
    # Track highest height for reporting
    if [[ "$height" =~ ^[0-9]+$ ]] && [ $height -gt $highest_height ]; then
      highest_height=$height
    fi
    
    if ! [[ "$height" =~ ^[0-9]+$ ]] || [ $height -lt $min_height ]; then
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

last_seen_consensus_version=0
last_seen_height=0
# Function checking that the first node reached the latest (unchanging) consensus version.
consensus_version_stable() {
  consensus_version=$(curl -s "http://localhost:3030/$network_name/consensus_version")
  height=$(curl -s "http://localhost:3030/$network_name/block/height/latest")
  if [[ "$consensus_version" =~ ^[0-9]+$ ]] && [[ "$height" =~ ^[0-9]+$ ]]; then
    # If the consensus version is greater than the last seen, we update it.
    if [[ "$consensus_version" -gt "$last_seen_consensus_version" ]]; then
      echo "✅ Consensus version updated to $consensus_version"
    # If the consensus version is the same whereas the block height is different and at least 10, we can assume that the consensus version is stable
    else
      if [[ "$height" -ne "$last_seen_height" ]] && [[ "$height" -ge 10 ]]; then
        echo "✅ Consensus version is stable at $consensus_version with height $height"
        return 0
      fi
    fi
  else
    echo "❌ Failed to retrieve consensus version or height: $consensus_version, $height"
    exit 1
  fi
  last_seen_consensus_version=$consensus_version
  last_seen_height=$height
  return 1
}

# Check consensus versions periodically with a timeout
total_wait=0
while [ $total_wait -lt 300 ]; do  # 5 minutes max
  if consensus_version_stable; then
    break    
  fi
  
  # Continue waiting
  sleep 30
  total_wait=$((total_wait + 30))
  echo "Waited $total_wait seconds so far..."
done

# Function checking that nodes created logs on disk.
check_logs() {
  echo "Checking logs for all nodes..."
  all_reached=true
  for ((validator_index = 0; validator_index < $total_validators; validator_index++)); do
    if [ ! -s "$log_dir/validator-${validator_index}.log" ]; then
      echo "❌ Test failed! Validator #${validator_index} did not create any logs."
      ls $log_dir
      return 1
    fi
  done
  for ((client_index = 0; client_index < $total_clients; client_index++)); do
    if [ ! -s "$log_dir/client-${client_index}.log" ]; then
      echo "❌ Test failed! Client #${client_index} did not create any logs."
      ls $log_dir
      return 1
    fi
  done

  return 0
}

# Deploy a program.
mkdir -p program
echo """program test_program.aleo;

function main:
    input r0 as u32.public;
    input r1 as u32.private;
    add r0 r1 into r2;
    output r2 as u32.private;

constructor:
    assert.eq true true;
""" > program/main.aleo
echo """{
  \"program\": \"test_program.aleo\",
  \"version\": \"0.1.0\",
  \"description\": \"\",
  \"license\": \"\",
  \"dependencies\": null,
  \"editions\": {}
}
""" > program/program.json
cd program
# Deploy the program.
deploy_result=$(snarkos developer deploy --private-key APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH --network $network_id --priority-fee 0 --broadcast http://localhost:3030/$network_name/transaction/broadcast --query http://localhost:3030 test_program.aleo)
# Wait for the deployment to be processed.
sleep 10
# Execute a function in the deployed program.
execute_result=$(snarkos developer execute --private-key APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH --network $network_id --query http://localhost:3030 --broadcast http://localhost:3030/$network_name/transaction/broadcast test_program.aleo main 1u32 1u32)
# Wait for the execution to be processed.
sleep 10
# Fail if the execution transaction does not exist.
tx=$(echo "$execute_result" | tail -n 1)
found=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3030/$network_name/transaction/$tx")
# Fail if the HTTP response is not 2XX.
if [[ "$found" -lt 200 || "$found" -ge 300 ]]; then
  echo "❌ Test failed! Transaction does not exist or contains an error: \ndeploy_result: $deploy_result\n\execute_result: $execute_result\nfound: $found"
  exit 1
else
  echo "✅ Transaction executed successfully: $execute_result"
fi
# Scan the network for records.
scan_result=$(snarkos developer scan --private-key APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH --network $network_id --start 0 --endpoint http://localhost:3030)
num_records=$(echo "$scan_result" | grep "owner" | wc -l)
# Fail if the scan did not return 4 records.
if [[ "$num_records" -ne 4 ]]; then
  echo "❌ Test failed! Expected 4 records, but found $num_records: $scan_result"
  exit 1
else
  echo "✅ Scan returned 4 records correctly: $scan_result"
fi

# Check heights periodically with a timeout
total_wait=0
while [ $total_wait -lt 600 ]; do  # 10 minutes max
  if check_heights; then
    echo "🎉 Test passed! All nodes reached minimum height."
    
    if check_logs; then
      exit 0
    else
      exit 1
    fi
  fi
  
  # Continue waiting
  sleep 30
  total_wait=$((total_wait + 30))
  echo "Waited $total_wait seconds so far..."
done

echo "❌ Test failed! Not all nodes reached minimum height within 15 minutes."

# Print logs for debugging
echo "Last 20 lines of validator logs:"
for ((validator_index = 0; validator_index < $total_validators; validator_index++)); do
  echo "=== Validator $validator_index logs ==="
  tail -n 20 "$log_dir/validator-$validator_index.log"
done

exit 1