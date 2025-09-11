#!/bin/bash

###########################################################
# Measures a client syncing 1000 blocks from another client
###########################################################

set -eo pipefail # error on any command failure

network_id=1
min_height=250

# The number of clients that are syncing
num_clients=1

# Adjust this to show more/less log messages
log_filter="info,snarkos_node_sync=debug,snarkos_node_tcp=warn,snarkos_node_rest=warn"

max_wait=2400 # Wait for up to 40 minutes
poll_interval=1 # Check block heights every second

. ./.ci/utils.sh

# Running sums for variance: use sum and sumsq for unbiased sample variance
sum_speed=0
sumsq_speed=0
samples=0
max_speed=0.0

# Fetch sync speeds from clients via REST and accumulate stats
function sample_sync_speeds() {
  for ((client_index = 1; client_index <= num_clients; client_index++)); do
    port=$((3030 + client_index))
    resp=$(curl -s "http://127.0.0.1:$port/$network_name/sync/status" || true)

    # Skip if response missing
    if [[ -z "$resp" ]]; then
      continue
    fi

    speed=$(echo "$resp" | jq -r '.sync_speed_bps')

    # Skip null or empty
    if [[ -z "$speed" ]] || [[ "$speed" == "null" ]]; then
      echo "Invalid speed value $speed"
      continue
    fi

    # Validate numeric (allow exponent)
    if ! (is_float "$speed"); then
        echo "Invalid speed value $speed"
       continue
    fi

    # Convert to fixed decimal for bc -l
    speed_dec=$(awk -v x="$speed" 'BEGIN{printf "%.12f", x}')
    if [[ -z "$speed_dec" ]]; then
      continue
    fi

    if (( $(echo "$speed > $max_speed" | bc -l) )); then
      max_speed=$speed
    fi

    # Accumulate using bc -l for floating point
    sum_speed=$(echo "$sum_speed + $speed_dec" | bc -l)
    sumsq_speed=$(echo "$sumsq_speed + ($speed_dec * $speed_dec)" | bc -l)
    samples=$((samples + 1))
  done
}

function write_rest_results() {
  local name=$1
  local num_ops=$2
  local total_wait=$3
  local endpoint=$4

  local throughput=$(compute_throughput "$num_ops" "$total_wait")

  echo "🎉 REST benchmark \"$name\" done! It took $total_wait seconds for $num_ops ops. Throughput was $throughput ops/s."

  printf "{ \"name\": \"rest-$name\", \"unit\": \"ops/s\", \"value\": %.6f, \"extra\": \"num_ops=%i, total_wait=%i, endpoint=%s, %s\" },\n" \
       "$throughput" "$num_ops" "$total_wait" "$endpoint" "$snapshot_info" | tee -a results.json
}

# Measure how long it takes to get the node's current block height.
# This should not create much work on the snarkVM-side of things and is a good baseline# for how fast the REST API can be.
function measure_rest_block_height() {
  local num_warmup_ops=100
  local num_ops=10000

  url="http://localhost:3030/v2/$network_name/block/height/latest"

  for _ in $(seq "$num_warmup_ops"); do
    curl -f "$url" -s -o /dev/null
  done

  SECONDS=0
  for _ in $(seq $num_ops); do
    curl -f "$url" -s -o /dev/null
  done

  local total_wait=$SECONDS
  write_rest_results "block-height" "$num_ops" "$total_wait" "$url" 
}

# Measure how long it takes to get a random block.
function measure_rest_get_block() {
  local num_warmup_ops=10
  local num_get_ops=500

  base_url="http://localhost:3030/v2/$network_name/block"

  for _ in $(seq "$num_warmup_ops"); do
    height=$((RANDOM % min_height)) 
    curl -f "$base_url/$height" -s -o /dev/null
  done

  SECONDS=0
  for _ in $(seq $num_get_ops); do
    height=$((RANDOM % min_height))
    curl -f "$base_url/$height" -s -o /dev/null
  done

  local total_wait=$SECONDS
  write_rest_results "get-block" "$num_get_ops" "$total_wait" "$base_url"
}

branch_name=$(git rev-parse --abbrev-ref HEAD)
echo "On branch: ${branch_name}"

network_name=$(get_network_name $network_id)
echo "Using network: $network_name (ID: $network_id)"

snapshot_info=$(<info.txt)
echo "Snapshot_info: ${snapshot_info}"

# Create log directory
log_dir=".logs-$(date +"%Y%m%d%H%M%S")"
mkdir -p "$log_dir"

# Define a trap handler that cleans up all processes on exit.
trap stop_nodes EXIT
trap child_exit_handler CHLD

# Define a trap handler that prints a message when an error occurs.
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Shared flags betwen all nodes
common_flags=(
  --nodisplay --nobanner --noupdater # reduce clutter in the output
  "--log-filter=$log_filter" # only show the logs we care about
  "--network=$network_id"
  --nocdn # don't sync from CDN, so we only benchmark p2p sync
  --dev-num-validators=40 --no-dev-txs
  --rest-rps=1000000 # ensure benchmarks don't fail due to rate limiting
)

# The client that has the ledger
# (runs on the first two cores)
$TASKSET1 snarkos start --dev 0 --client "${common_flags[@]}" \
  --logfile="$log_dir/client-0.log" &
PIDS[0]=$!

# Spawn the clients that will sync the ledger
# (running on the other two cores)
for client_index in $(seq 1 "$num_clients"); do
  prev_port=$((4130+client_index-1))
  name="client-$client_index"

  # Ensure there are no old ledger files and the node syncs from scratch
  snarkos clean "--dev=$client_index" "--network=$network_id" || true

  $TASKSET2 snarkos start "--dev=$client_index" --client \
    "${common_flags[@]}" "--peers=127.0.0.1:$prev_port" \
    "--logfile=$log_dir/$name.log" &
  PIDS[client_index]=$!

  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Block until nodes are running and connected to each other.
wait_for_nodes 0 $((num_clients+1))

# It takes about 30s for nodes to connect. Do not measure this time.
SECONDS=0
for node_index in $(seq 0 "$num_clients"); do
  if ! (wait_for_peers "$node_index" $num_clients); then
    exit 1
  fi
done

connect_time=$SECONDS
echo "ℹ️ Nodes are fully connected (took $connect_time secs). Starting block sync measurement."

# Check heights periodically with a timeout
SECONDS=0
while (( SECONDS < max_wait )); do
  # Sample sync speed(s) for variance calculation
  sample_sync_speeds

  if check_heights 1 $((num_clients+1)) $min_height "$network_name" "$SECONDS"; then
    total_wait=$SECONDS
    throughput=$(compute_throughput "$min_height" "$total_wait")

    # Compute unbiased sample variance of sync_speed_bps (in blocks^2/s^2)
    if (( samples > 1 )); then
      mean_speed=$(echo "scale=8; $sum_speed / $samples" | bc -l)
      variance=$(echo "scale=8; (($sumsq_speed / $samples) - ($mean_speed * $mean_speed)) * ($samples / ($samples - 1))" | bc -l)
    else
      mean_speed=$(echo "scale=8; 0" | bc -l)
      variance=$(echo "scale=8; 0" | bc -l)
    fi

    echo "🎉 P2P sync benchmark done! Waited $total_wait seconds for $min_height blocks. Throughput was $throughput blocks/s."

    # Append data to results file.
    printf "{ \"name\": \"p2p-sync\", \"unit\": \"blocks/s\", \"value\": %.3f, \"extra\": \"total_wait=%is, target_height=%i, connect_time=%is, %s\" },\n" \
       "$throughput" "$total_wait" "$min_height" "$connect_time" "$snapshot_info" | tee -a results.json
    printf "{ \"name\": \"p2p-sync-speed-variance\", \"unit\": \"blocks^2/s^2\", \"value\": %.6f, \"extra\": \"samples=%d, mean_speed=%.6f, max_speed=%.6f, branch=%s, %s\" },\n" \
       "$variance" "$samples" "$mean_speed" "$max_speed" "$branch_name" "$snapshot_info" | tee -a results.json

    measure_rest_get_block
    measure_rest_block_height

    exit 0
  fi
  
  # Continue waiting
  sleep $poll_interval
done

echo "❌ Benchmark failed! Clients did not sync within 40 minutes."

# Print logs for debugging
echo "Last 20 lines of client logs:"
for ((client_index = 0; client_index < num_clients; client_index++)); do
  echo "=== Client $client_index logs ==="
  tail -n 20 "$log_dir/client-$client_index.log"
done

exit 1
