#!/bin/bash

####################################################
# Runs a devnet in which one validator lacks the next consensus version.
# Checks that the outdated validator exits with an error and without forking, both while it runs
# and on every restart, with or without its record of the required upgrade. Then checks that it
# rejoins the network once upgraded.
####################################################

set -eo pipefail

network_id=0
total_validators=4
activation_height=150
target_height=180
# The outdated validator is the last one, so that the upgraded ones form a contiguous range.
outdated_validator=$((total_validators-1))
NODE_VERBOSITY=2

# shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh

network_name=$(get_network_name "$network_id")
init_log_dir

trap stop_nodes EXIT

# The heights of V1..V21 match the test defaults; only V22 differs.
base_heights="0,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24"
upgraded_heights="$base_heights,$activation_height"
outdated_heights="$base_heights,4294967295"

common_flags=(
  --nodisplay --nobanner --noupdater "--network=$network_id" "--verbosity=$NODE_VERBOSITY"
  "--dev-num-validators=$total_validators" "--dev-num-clients=0"
)

upgraded_port=3030
outdated_port=$((3030+outdated_validator))
record_file=".node-data-$network_id-$outdated_validator/required-consensus-upgrade"

# Starts validator $1 with the consensus version heights $2, logging to $3.
function start_validator() {
  local index=$1
  local heights=$2
  local log_file=$3
  CONSENSUS_VERSION_HEIGHTS=$heights run_with_prefix "validator-$index" snarkos start "${common_flags[@]}" \
    "--dev=$index" --validator "--logfile=$log_file" "--rest=127.0.0.1:$((3030+index))" --no-dev-txs
  PIDS[index]=$!
}

# Prints the hash of the block at height $2 on the node with the REST port $1, or nothing.
function block_hash() {
  curl -s --max-time 5 "http://127.0.0.1:$1/v2/$network_name/block/$2" | jq -r '.block_hash // empty' 2>/dev/null || true
}

# Fails unless the upgraded validators hold the block with hash $2 at height $1.
function check_no_fork() {
  local height=$1
  local outdated_hash=$2
  local upgraded_hash
  upgraded_hash=$(block_hash "$upgraded_port" "$height")
  if [[ -z "$outdated_hash" || "$outdated_hash" != "$upgraded_hash" ]]; then
    log "⛔️ At height $height, the outdated validator holds '$outdated_hash' and the upgraded ones '$upgraded_hash'"
    exit 1
  fi
}

# Waits for the outdated validator to exit, and checks that it exited with an error, recorded the
# required upgrade, and holds no block the upgraded validators lack. $1 names the run, $2 is the
# height the outdated validator must stay below, if any, and $3 is its log file.
function expect_outdated_exit() {
  local run=$1
  local max_height=$2
  local log_file=$3
  local pid=${PIDS[outdated_validator]}
  local height last_height="" last_hash="" hash start
  start=$(now)

  while kill -0 "$pid" 2>/dev/null; do
    if (( $(elapsed_since "$start") > 600 )); then
      log "⛔️ The outdated validator did not exit ($run)"
      exit 1
    fi
    height=$(get_block_height_by_port "$outdated_port" "$network_name" 5)
    if [[ -n "$height" ]]; then
      hash=$(block_hash "$outdated_port" "$height")
      if [[ -n "$hash" ]]; then
        last_height=$height
        last_hash=$hash
      fi
    fi
    sleep 1
  done

  local status=0
  wait "$pid" || status=$?
  log "The outdated validator exited with status $status at height '$last_height' ($run)"
  if (( status == 0 )); then
    log "⛔️ The outdated validator exited without an error ($run)"
    exit 1
  fi
  if [[ ! -f "$record_file" ]]; then
    log "⛔️ The outdated validator did not record the required upgrade ($run)"
    exit 1
  fi
  if ! grep -q "must be upgraded" "$log_file"; then
    log "⛔️ The outdated validator did not log that it must be upgraded ($run)"
    exit 1
  fi
  if [[ -z "$last_height" ]]; then
    log "⛔️ The outdated validator never reported its height ($run)"
    exit 1
  fi
  if [[ -n "$max_height" ]] && (( last_height >= max_height )); then
    log "⛔️ The outdated validator reached height $last_height ($run)"
    exit 1
  fi
  check_no_fork "$last_height" "$last_hash"
}

for validator_index in $(seq 0 $((total_validators-1))); do
  snarkos clean "--dev=$validator_index" "--network=$network_id"
  if (( validator_index == outdated_validator )); then
    heights=$outdated_heights
  else
    heights=$upgraded_heights
  fi
  start_validator "$validator_index" "$heights" "$log_dir/validator-$validator_index.log"
  sleep 1
done

wait_for_nodes "$total_validators" 0 "$network_name" 180

# The outdated validator detects the upgrade while it runs, before the activation height.
expect_outdated_exit "first run" "$activation_height" "$log_dir/validator-$outdated_validator.log"

# The upgraded validators continue without it.
if ! wait_for_heights 0 "$outdated_validator" "$target_height" "$network_name" 1800 5; then
  log "⛔️ Upgraded validators did not reach height $target_height"
  exit 1
fi
log "Upgraded validators reached height $target_height"

# A restart with the same build and the record exits again.
log_file="$log_dir/validator-$outdated_validator-restart.log"
start_validator "$outdated_validator" "$outdated_heights" "$log_file"
expect_outdated_exit "restart with the record" "" "$log_file"
if ! grep -q "A previous run found" "$log_file"; then
  log "⛔️ The restarted validator did not read the record"
  exit 1
fi

# A start with the same build and no record exits too, as peers keep announcing the version they run.
rm "$record_file"
log_file="$log_dir/validator-$outdated_validator-fresh.log"
start_validator "$outdated_validator" "$outdated_heights" "$log_file"
expect_outdated_exit "start without the record" "" "$log_file"

# Once upgraded, the validator removes the record and rejoins.
log_file="$log_dir/validator-$outdated_validator-upgraded.log"
start_validator "$outdated_validator" "$upgraded_heights" "$log_file"
rejoin_height=$(( $(get_block_height_by_port "$upgraded_port" "$network_name" 5) + 10 ))
if ! wait_for_heights "$outdated_validator" "$total_validators" "$rejoin_height" "$network_name" 900 5; then
  log "⛔️ The upgraded validator did not reach height $rejoin_height"
  exit 1
fi
if [[ -f "$record_file" ]]; then
  log "⛔️ The upgraded validator kept the record"
  exit 1
fi
check_no_fork "$rejoin_height" "$(block_hash "$outdated_port" "$rejoin_height")"

# A record of a version that no build schedules is cleared by a quorum of validators that are not ahead.
graceful_stop_pid "${PIDS[outdated_validator]}" "validator-$outdated_validator"
# Height 1000 and ConsensusVersion 65535, in the little-endian encoding of the record.
printf '\xe8\x03\x00\x00\xff\xff' > "$record_file"
log_file="$log_dir/validator-$outdated_validator-forged.log"
start_validator "$outdated_validator" "$upgraded_heights" "$log_file"
clear_height=$(( $(get_block_height_by_port "$upgraded_port" "$network_name" 5) + 10 ))
if ! wait_for_heights "$outdated_validator" "$total_validators" "$clear_height" "$network_name" 900 5; then
  log "⛔️ The validator with a forged record did not reach height $clear_height"
  exit 1
fi
if ! grep -q "A previous run found" "$log_file" || ! grep -q "Connected validators holding a quorum" "$log_file"; then
  log "⛔️ The validator did not read and then clear the forged record"
  exit 1
fi
if [[ -f "$record_file" ]]; then
  log "⛔️ The validator kept the forged record"
  exit 1
fi
check_no_fork "$clear_height" "$(block_hash "$outdated_port" "$clear_height")"
if check_node_stopped; then
  exit 1
fi

log "✅ The outdated validator exited without forking on every run, rejoined once upgraded, and cleared a forged record"
