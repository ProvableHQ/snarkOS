#!/bin/bash

# Set parameters directly
total_validators=$1
total_clients=$2
network_id=$3
min_height=$4

# Default values if not provided
: "${total_validators:=4}"
: "${total_clients:=2}"
: "${network_id:=0}"
: "${min_height:=45}"

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
log_dir=".logs-$(date +"%Y%m%d%H%M%S")"
mkdir -p "$log_dir"

# Array to store PIDs of all processes
declare -a PIDS

# Start all validator nodes in the background
for ((validator_index = 0; validator_index < $total_validators; validator_index++)); do
  log_file="$log_dir/validator-$validator_index.log"
  if [ "$validator_index" -eq 0 ]; then
    snarkos start --nodisplay --network $network_id --dev $validator_index --allow-external-peers --dev-num-validators $total_validators --validator --logfile $log_file --metrics &
  else
    snarkos start --nodisplay --network $network_id --dev $validator_index --allow-external-peers --dev-num-validators $total_validators --validator --logfile $log_file &
  fi
  PIDS[$validator_index]=$!
  echo "Started validator $validator_index with PID ${PIDS[$validator_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Start all client nodes in the background
for ((client_index = 0; client_index < $total_clients; client_index++)); do
  node_index=$((client_index + total_validators))
  log_file="$log_dir/client-$client_index.log"
  snarkos start --nodisplay --network $network_id --dev $node_index --dev-num-validators $total_validators --client --logfile $log_file &
  PIDS[$node_index]=$!
  echo "Started client $client_index with PID ${PIDS[$node_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  if [ $client_index -lt $((total_clients - 1)) ]; then
    sleep 1
  fi
done

# Function to check block heights
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

# Wait for 60 seconds to let the network start up
echo "Waiting 60 seconds for network to start up..."
sleep 60

# Check heights periodically with a timeout
total_wait=0
while [ $total_wait -lt 900 ]; do  # 15 minutes max
  if check_heights; then
    echo "🎉 Test passed! All nodes reached minimum height."
    
    # Cleanup: kill all processes
    for pid in "${PIDS[@]}"; do
      kill -9 $pid 2>/dev/null || true
    done
    
    exit 0
  fi
  
  # Continue waiting
  sleep 60
  total_wait=$((total_wait + 60))
  echo "Waited $total_wait seconds so far..."
done

echo "❌ Test failed! Not all nodes reached minimum height within 15 minutes."

# Print logs for debugging
echo "Last 20 lines of validator logs:"
for ((validator_index = 0; validator_index < $total_validators; validator_index++)); do
  echo "=== Validator $validator_index logs ==="
  tail -n 20 "$log_dir/validator-$validator_index.log"
done

# Cleanup: kill all processes
for pid in "${PIDS[@]}"; do
  kill -9 $pid 2>/dev/null || true
done

exit 1