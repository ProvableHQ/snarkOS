#!/bin/bash

####################################################
# Runs and tests a development network
####################################################

set -eo pipefail # error on any command failure

# Uncomment this to print commands before executing them for easier debugging.
#set -x

# Set parameters directly
total_validators=$1
total_clients=$2
network_id=$3
min_height=$4

# The verobsity of snarkos nodes.
NODE_VERBOSITY=3

# Default values if not provided
: "${total_validators:=4}"
: "${total_clients:=4}" # need at least 4 clients, so each validator has at least one client connected to it.
: "${network_id:=0}"
: "${min_height:=60}" # To likely go past the 100 round garbage collection limit.

# shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh

# Determine network name based on network_id
network_name=$(get_network_name "$network_id")
echo "Using network: $network_name (ID: $network_id)"

# Create log directory
log_dir="$PWD/.logs-$(date +"%Y%m%d%H%M%S")"
mkdir -p "$log_dir"
chmod 755 "$log_dir"

# Ensures we use IPv4 localhost everywhere.
localhost="127.0.0.1"

# Define a trap handler that cleans up all processes on exit.
# shellcheck disable=SC2329
function exit_handler() {
  stop_nodes

  # Remove all temporary files and folders
  rm program/program.json program/main.aleo || true
  rm program/txn_data.json program/invalid_txn_data.json || true
  rmdir program || true
}
trap exit_handler EXIT

# Define a trap handler that prints a message when an error occurs 
trap 'echo "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Flags used by all nodes.
common_flags=(
  --nodisplay --nobanner --noupdater "--network=$network_id" "--verbosity=$NODE_VERBOSITY"
  "--dev-num-validators=$total_validators"  "--dev-num-clients=$total_clients"
)

# Start all validator nodes in the background
for ((validator_index = 0; validator_index < total_validators; validator_index++)); do
  snarkos clean "--dev=$validator_index" "--network=$network_id"

  log_file="$log_dir/validator-$validator_index.log"
  if [ $validator_index -eq 0 ]; then
    snarkos start "${common_flags[@]}" "--dev=$validator_index" \
      --validator "--logfile=$log_file" --metrics --no-dev-txs &
  else
    snarkos start "${common_flags[@]}" "--dev=$validator_index" \
      --validator "--logfile=$log_file" &
  fi
  PIDS[validator_index]=$!
  echo "Started validator $validator_index with PID ${PIDS[$validator_index]}"

  # Add 1-second delay between starting nodes to avoid hitting rate limits
  sleep 1
done

# Start all client nodes in the background.
for ((client_index = 0; client_index < total_clients; client_index++)); do
  # compute the absolute index for this node.
  node_index=$((client_index + total_validators))

  snarkos clean "--dev=$node_index" "--network=$network_id"

  log_file="$log_dir/client-$client_index.log"
  snarkos start "${common_flags[@]}" "--dev=$node_index" \
    --client "--logfile=$log_file" &
  PIDS[node_index]=$!
  echo "Started client $client_index with PID ${PIDS[$node_index]}"
  # Add 1-second delay between starting nodes to avoid hitting rate limits
  if (( client_index < total_clients-1)); then
    sleep 1
  fi
done

# Ensure all nodes are up and running.
wait_for_nodes "$total_validators" "$total_clients"

# Ensure all nodes have at least one peer
echo "ℹ️ Waiting for all nodes to have at least one peer..."
SECONDS=0
for node_index in $(seq 0 $((total_clients+total_validators))); do
  if ! (wait_for_peers "$node_index" 1); then
    exit 1
  fi
done
echo "✅ All nodes have at least one peer"

last_seen_consensus_version=0
last_seen_height=0

# Function checking that the first node reached the latest (unchanging) consensus version.
function consensus_version_stable() {
  consensus_version=$(curl -s "http://$localhost:3030/v2/$network_name/consensus_version")
  height=$(curl -s "http://$localhost:3030/v2/$network_name/block/height/latest")

  if (! is_integer "$consensus_version"); then
    echo "❌ Failed to retrieve consensus version: $consensus_version"
    return 1
  elif (! is_integer "$height"); then
    echo "❌ Failed to retrieve height: $height"
    return 1
  else
    # If the consensus version is greater than the last seen, we update it.
    if (( consensus_version > last_seen_consensus_version )); then
      echo "✅ Consensus version updated to $consensus_version"
    # If the consensus version is the same whereas the block height is different and at least 10, we can assume that the consensus version is stable
    else
      if (( (height != last_seen_height) && (height >= 10) )); then
        echo "✅ Consensus version is stable at $consensus_version with height $height"
        return 0
      fi
    fi

    last_seen_consensus_version=$consensus_version
    last_seen_height=$height
  fi

  return 1
}

# Check consensus versions periodically with a timeout
echo "ℹ️ Waiting for consensus version to stabilize..."
total_wait=0
version_stable=false
while (( total_wait < 300 )); do  # 5 minutes max
  if consensus_version_stable; then
    version_stable=true
    break
  fi

  # Continue waiting
  sleep 30
  total_wait=$((total_wait + 30))
  echo "Waited $total_wait seconds so far..."
done

if ! $version_stable; then
  echo "❌ Test failed! Consensus version did not stabilize within 5 minutes."
  exit 1
fi

# Creates a test program.
mkdir -p program
program_name="test_program.aleo"
echo "program ${program_name};

function main:
    input r0 as u32.public;
    input r1 as u32.private;
    add r0 r1 into r2;
    output r2 as u32.private;

constructor:
    assert.eq true true;
" > program/main.aleo

echo "{
  \"program\": \"${program_name}\",
  \"version\": \"0.1.0\",
  \"description\": \"\",
  \"license\": \"\",
  \"dependencies\": null,
  \"editions\": {}
}
" > program/program.json

# Deploy the test program and wait for the deployment to be processed.
echo "● Testing program deployment..."
_deploy_result=$(cd program && snarkos developer deploy --dev-key 0 --network "$network_id" --endpoint=localhost:3030 --broadcast --wait --timeout 20 "$program_name")

# Ensure we are able to fetch the program from the node.
status_code=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3030/v2/$network_name/program/${program_name}/0")
if (( status_code == 200 )); then
  echo "✅ Program exists on the node"
else
  echo "❌ Test failed! Failed to get program. Code was ${status_code}"
  exit 1
fi

# Ensure the latest edition is indeed 0.
echo "● Testing retrieval of program editions..."
edition=$(curl -s -o /dev/null "http://localhost:3030/v2/$network_name/program/${program_name}/latest_edition")
if (( edition != 0 )); then
  echo "❌ Test failed! Invalid latest edition {} for test program returned, not 0."
  exit 1
fi

# Also check that the latest edition for the default program (credits.aleo) is 0.
edition=$(curl -s -o /dev/null "http://localhost:3030/v2/$network_name/program/credits.aleo/latest_edition")
if (( edition != 0 )); then
  echo "❌ Test failed! Invalid latest edition {} for credits.aleo returned, not 0."
  exit 1
fi

# Finally, check that we cannot fetch a non-existing edition of a program
status_code=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3030/v2/$network_name/program/${program_name}/1")
if (( status_code == 404 )); then
  echo "✅ Only program edition 0 exists on the node"
else
  echo "❌ Test failed! Invalid edition returnd ${status_code}, not 404."
  exit 1
fi

# Execute a function in the deployed program and wait for the execution to be processed.
echo "● Testing program execution with V2 API..."
execute_result=$(cd program && snarkos developer execute --dev-key 0 --network "$network_id" --broadcast --endpoint=http://localhost:3030 \
    "$program_name" main 1u32 1u32 --wait --timeout 10)

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

# Use the old flags here `--query` and `--broadcast=URL` to test they still work.
# Also, use the v1 API to test it still works.
echo "● Testing program execution with V1 API..."
execute_result=$(cd program && snarkos developer execute --dev-key 0 --network "$network_id" --query=http://$localhost:3030/v1 \
    "--broadcast=http://$localhost:3030/v1/$network_name/transaction/broadcast" "$program_name" main 1u32 1u32 --wait --timeout 10)

# Fail if the execution transaction does not exist.
tx=$(echo "$execute_result" | tail -n 1)
found=$(curl -s -o /dev/null -w "%{http_code}" "http://$localhost:3030/v1/$network_name/transaction/$tx")
# Fail if the HTTP response is not 2X.
if (( found < 200 || found >= 300 )); then
  printf "❌ Test failed! Transaction does not exist or contains an error: \nexecute_result: %s\nfound: %s\n" \
    "$execute_result" "$found"
  exit 1
else
  echo "✅ Transaction executed successfully: $execute_result"
fi

# Fail if status does not exist or is not set to "accepted".
echo "● Testing confirmed transaction endpoint..."
rest_confirmed=$(curl -s "http://$localhost:3030/v2/$network_name/transaction/confirmed/$tx")

rest_status=$(jq --raw-output '.status' <<< "$rest_confirmed")
if [ "$rest_status" != "accepted" ]; then
  printf "❌ Test failed! Rest API did not mark the transaction as \"accepted\". Status was: \"%s\" \nFull JSON: %s\n" "$rest_status" "$rest_confirmed"
  exit 1
fi

echo "ℹ️Testing REST API and REST Error Handling"

# Test invalid transaction data (JsonDataError) returns 422 Unprocessable Content
echo "● Testing invalid transaction data returns 422 status code..."
(cd program && snarkos developer execute --dev-key 0 --network "$network_id" \
  "--endpoint=$localhost:3030"  --store txn_data.json --store-format=string \
  "$program_name" main 1u32 1u32)

# Modify the proof data
# This changes the last three characters in the hash but keeps the correct length.
# `printf %s` avoids a newline at the end.
(cd program && printf %s "$(jq -c '.id = (.id[0:-3] + "qpz")' txn_data.json)" > invalid_txn_data.json)

invalid_tx_status=$(curl -s -w "%{http_code}" -X POST \
  -H "Content-Type: application/json" \
  -d "$(< ./program/invalid_txn_data.json)" \
  "http://$localhost:3030/v2/$network_name/transaction/broadcast" \
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
  "http://$localhost:3030/v2/$network_name/transaction/broadcast")

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
  "http://$localhost:3030/v2/$network_name/transaction/broadcast" \
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
  "http://$localhost:3030/v2/$network_name/transaction/broadcast")

# Verify the message contains JSON-related error text
if ! jq -e '.message | test("Invalid JSON")' <<< "$malformed_json_error" > /dev/null; then
  echo "❌ Test failed! Malformed JSON response message doesn't contain expected JSON error text: \"$malformed_json_error\""
  exit 1
fi

echo "✅ Malformed JSON returns properly formatted RestError with JSON syntax error message"

# Test invalid Content-Type header returns 400 Bad Request
echo "● Testing missing Content-Type header returns 400 status code..."
missing_content_type_response=$(curl -s -w "%{http_code}" -X POST \
  -d '{"valid": "json"}' \
  "http://$localhost:3030/v2/$network_name/transaction/broadcast" \
  -o /dev/null)

if (( missing_content_type_response == 400 )); then
  echo "✅ Missing Content-Type correctly returned 400 Bad Request"
else
  echo "❌ Test failed! Missing Content-Type returned $missing_content_type_response instead of 400"
  exit 1
fi

# Test that missing Content-Type returns a properly formatted RestError
echo "● Testing missing Content-Type returns valid RestError format..."

missing_content_type_error=$(curl -s -X POST \
  -d '{"valid": "json"}' \
  "http://$localhost:3030/v2/$network_name/transaction/broadcast")

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
echo "● Testing \`snarkos developer scan\`..."

scan_result=$(snarkos developer scan --dev-key 0 --network "$network_id" --start 0 "--endpoint=$localhost:3030")
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
  if check_heights 0 $((total_validators+total_clients)) "$min_height" "$network_name" "$total_wait"; then
    echo "🎉 Test passed! All nodes reached minimum height."

    if check_logs "$log_dir" "$total_validators" "$total_clients"; then
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
print_validator_logs "$log_dir" "$total_validators" "$total_clients"
print_client_logs "$log_dir" "$total_validators" "$total_clients"

exit 1
