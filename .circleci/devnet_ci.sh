#!/bin/bash
set -eo pipefail # error on any command failure

# Uncomment this to print commands before executing them for easier debugging.
#set -x

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

# Change this to increase/decrease log messages
log_filter="info,snarkos_node_router=error,snarkos_node_tcp=error"

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
log_dir="$PWD/.logs-$(date +"%Y%m%d%H%M%S")"
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

  # Remove all temporary files and folders
  rm program/program.json program/main.aleo || true
  rm program/txn_data.json program/invalid_txn_data.json || true
  rmdir program || true
}
trap cleanup EXIT
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Flags used by all ndoes
common_flags="--nodisplay --nobanner --noupdater --network=$network_id \
  --log-filter=$log_filter \
  --dev-num-validators=$total_validators"

# Start all validator nodes in the background
for ((validator_index = 0; validator_index < total_validators; validator_index++)); do
  snarkos clean --dev $validator_index

  log_file="$log_dir/validator-$validator_index.log"
  if [ $validator_index -eq 0 ]; then
    snarkos start ${common_flags} --dev "$validator_index" --allow-external-peers \
      --validator --logfile "$log_file" \
      --metrics --no-dev-txs &
  else
    snarkos start ${common_flags} --dev "$validator_index" --allow-external-peers \
      --validator --logfile "$log_file" &
  fi
  PIDS[validator_index]=$!
  echo "Started validator $validator_index with PID ${PIDS[$validator_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Start all client nodes in the background
for ((client_index = 0; client_index < total_clients; client_index++)); do
  node_index=$((client_index + total_validators))

  snarkos clean --dev $node_index

  log_file="$log_dir/client-$client_index.log"
  snarkos start ${common_flags} --dev $node_index \
    --client --logfile "$log_file" &
  PIDS[node_index]=$!
  echo "Started client $client_index with PID ${PIDS[$node_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  if (( client_index < total_clients-1)); then
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

last_seen_consensus_version=0
last_seen_height=0

# Function checking that the first node reached the latest (unchanging) consensus version.
consensus_version_stable() {
  consensus_version=$(curl -s "http://localhost:3030/v2/$network_name/consensus_version")
  height=$(curl -s "http://localhost:3030/v2/$network_name/block/height/latest")
  if [[ "$consensus_version" =~ ^[0-9]+$ ]] && [[ "$height" =~ ^[0-9]+$ ]]; then
    # If the consensus version is greater than the last seen, we update it.
    if (( consensus_version > last_seen_consensus_version)); then
      echo "✅ Consensus version updated to $consensus_version"
    # If the consensus version is the same whereas the block height is different and at least 10, we can assume that the consensus version is stable
    else
      if (( (height != last_seen_height) && (height >= 10) )); then
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
while (( total_wait < 300 )); do  # 5 minutes max
  if consensus_version_stable; then
    break
  fi

  # Continue waiting
  sleep 30
  total_wait=$((total_wait + 30))
  echo "Waited $total_wait seconds so far..."
done

# Function checking that nodes created logs on disk.
function check_logs() {
  echo "Checking logs for all nodes..."
  for ((validator_index = 0; validator_index < total_validators; validator_index++)); do
    if [ ! -s "$log_dir/validator-${validator_index}.log" ]; then
      echo "❌ Test failed! Validator #${validator_index} did not create any logs."
      ls "$log_dir"
      return 1
    fi
  done
  for ((client_index = 0; client_index < total_clients; client_index++)); do
    if [ ! -s "$log_dir/client-${client_index}.log" ]; then
      echo "❌ Test failed! Client #${client_index} did not create any logs."
      ls "$log_dir"
      return 1
    fi
  done

  return 0
}

echo "ℹ️Testing Program Deployment and Execution"

# Creates a test program.
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

# Deploy the test program and wait for the deployment to be processed.
_deploy_result=$(cd program && snarkos developer deploy --dev-key 0 --network "$network_id" --endpoint=localhost:3030 --broadcast  --wait --timeout 10 test_program.aleo)

# Execute a function in the deployed program and wait for the execution to be processed.
# Use the old flags here `--query` and `--broadcast=URL` to test they still work.
# Also, use the v1 API to test it still works.
execute_result=$(cd program && snarkos developer execute --dev-key 0 --network "$network_id" --query=localhost:3030 --broadcast=http://localhost:3030/v1/${network_name}/transaction/broadcast test_program.aleo main 1u32 1u32 --wait --timeout 10)

# Fail if the execution transaction does not exist.
tx=$(echo "$execute_result" | tail -n 1)
found=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3030/v2/$network_name/transaction/$tx")
# Fail if the HTTP response is not 2XX.
if (( found < 200 || found >= 300 )); then
  printf "❌ Test failed! Transaction does not exist or contains an error: \nexecute_result: %s\nfound: %s\n" \
    "$execute_result" "$found"
  exit 1
else
  echo "✅ Transaction executed successfully: $execute_result"
fi

echo "ℹ️Testing REST API and REST Error Handling"

# Test invalid transaction data (JsonDataError) returns 422 Unprocessable Content
echo "Testing invalid transaction data returns 422 status code..."
(cd program && snarkos developer execute --dev-key 0 --network "$network_id" --endpoint=localhost:3030 \
  --store txn_data.json --store-format=string test_program.aleo main 1u32 1u32)

# Modify the proof data
# This changes the last three characters in the hash but keeps the correct length.
# `printf %s` avoids a newline at the end.
(cd program && printf %s "$(jq -c '.id = (.id[0:-3] + "qpz")' txn_data.json)" > invalid_txn_data.json)

invalid_tx_status=$(curl -s -w "%{http_code}" -X POST \
  -H "Content-Type: application/json" \
  -d "$(< ./program/invalid_txn_data.json)" \
  "http://localhost:3030/v2/$network_name/transaction/broadcast" \
  -o /dev/null)

if (( invalid_tx_status == 422 )); then
  echo "✅ Invalid transaction correctly returned 422 Unprocessable Content"
else
  echo "❌ Test failed! Invalid transaction returned $invalid_tx_status instead of 422"
  exit 1
fi

# Test that the returned error is valid JSON
json_error=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -d "$(< ./program/invalid_txn_data.json)" \
  "http://localhost:3030/v2/$network_name/transaction/broadcast")

# Ensure the top-level error message is "Invalid transaction"
if ! jq -e '.message | test("Invalid transaction")' <<< "$json_error" > /dev/null ; then 
  echo "❌ Test failed! Invalid JSON returned: \"$json_error\""
  exit 1
fi

echo "✅ Invalid transaction return valid JSON error"

# Test malformed JSON syntax (JsonSyntaxError) returns 400 Bad Request
malformed_json_response=$(curl -s -w "%{http_code}" -X POST \
  -H "Content-Type: application/json" \
  -d '{"malformed": json}' \
  "http://localhost:3030/v2/$network_name/transaction/broadcast" \
  -o /dev/null)

if (( malformed_json_response == 400 )); then
  echo "✅ Malformed JSON correctly returned 400 Bad Request"
else
  echo "❌ Test failed! Malformed JSON returned $malformed_json_response instead of 400"
  exit 1
fi

# Test that malformed JSON returns a properly formatted RestError
malformed_json_error=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -d '{"malformed": json}' \
  "http://localhost:3030/v2/$network_name/transaction/broadcast")

# Verify the message contains JSON-related error text
if ! jq -e '.message | test("Invalid JSON")' <<< "$malformed_json_error" > /dev/null; then
  echo "❌ Test failed! Malformed JSON response message doesn't contain expected JSON error text: \"$malformed_json_error\""
  exit 1
fi

echo "✅ Malformed JSON returns properly formatted RestError with JSON syntax error message"

# Test invalid Content-Type header returns 400 Bad Request
echo "Testing missing Content-Type header returns 400 status code..."
missing_content_type_response=$(curl -s -w "%{http_code}" -X POST \
  -d '{"valid": "json"}' \
  "http://localhost:3030/v2/$network_name/transaction/broadcast" \
  -o /dev/null)

if (( missing_content_type_response == 400 )); then
  echo "✅ Missing Content-Type correctly returned 400 Bad Request"
else
  echo "❌ Test failed! Missing Content-Type returned $missing_content_type_response instead of 400"
  exit 1
fi

# Test that missing Content-Type returns a properly formatted RestError
echo "Testing missing Content-Type returns valid RestError format..."
missing_content_type_error=$(curl -s -X POST \
  -d '{"valid": "json"}' \
  "http://localhost:3030/v2/$network_name/transaction/broadcast")

# Verify the response is valid JSON
if ! jq . <<< "$missing_content_type_error" > /dev/null 2>&1; then
  echo "❌ Test failed! Missing Content-Type response is not valid JSON: \"$missing_content_type_error\""
  exit 1
fi

# Verify the message contains Content-Type related error text
if ! jq -e '.message | test("Content-Type|application/json")' <<< "$missing_content_type_error" > /dev/null; then
  echo "❌ Test failed! Missing Content-Type response message doesn't contain expected error text: \"$missing_content_type_error\""
  exit 1
fi

echo "✅ Missing Content-Type returns properly formatted RestError with Content-Type error message"

# Scan the network for records.
scan_result=$(snarkos developer scan --dev-key 0 --network "$network_id" --start 0 --endpoint=localhost:3030)
num_records=$(echo "$scan_result" | grep -c "owner")
# Fail if the scan did not return 4 records.
if (( num_records != 4 )); then
  echo "❌ Test failed! Expected 4 records, but found $num_records: $scan_result"
  exit 1
else
  echo "✅ Scan returned 4 records correctly: $scan_result"
fi

echo "ℹ️Testing network progress"

# Check heights periodically with a timeout
total_wait=0
while (( total_wait < 600 )); do  # 10 minutes max
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
for ((validator_index = 0; validator_index < total_validators; validator_index++)); do
  echo "=== Validator $validator_index logs ==="
  tail -n 20 "$log_dir/validator-$validator_index.log"
done

exit 1
