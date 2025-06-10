#!/bin/bash

network_id=1
min_height=200
num_clients=4

. ./.circleci/utils.sh

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

# The client that has the ledger
log_file="$log_dir/client-0.log"
snarkos start --nodisplay --network $network_id --dev 0 --client --logfile $log_file &
PIDS[0]=$!

for ((client_index = 1; client_index < $num_clients; client_index++)); do
   # The clients that will sync the ledger
  prev_port=$((4130+$client_index-1))
  log_file="$log_dir/client-$client_index.log"
  snarkos start --nodisplay --network $network_id --dev $client_index --client --logfile $log_file --peers=127.0.0.1:$prev_port &
  PIDS[$client_index]=$!
  
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Wait for 60 seconds to let the network start up
echo "Waiting 60 seconds for network to start up..."
sleep 60

# Check heights periodically with a timeout
total_wait=0
while [ $total_wait -lt 300 ]; do  # 5 minutes max
  if check_heights 0 $num_clients $min_height; then
    echo "🎉 Test passed!."
    
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

echo "❌ Test failed! Clients did not sync within 5 minutes."

# Print logs for debugging
echo "Last 20 lines of client logs:"
for ((client_index = 0; client_index < $total_validators; client_index++)); do
  echo "=== Client $client_index logs ==="
  tail -n 20 "$log_dir/validator-$client_index.log"
done

# Cleanup: kill all processes
for pid in "${PIDS[@]}"; do
  kill -9 $pid 2>/dev/null || true
done

exit 1
