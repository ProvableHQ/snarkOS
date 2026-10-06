#!/bin/bash

####################################################
# Runs a devnet in which one validator lacks the next consensus version.
# Checks that it goes dark before the activation height, and that the rest continue.
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

for validator_index in $(seq 0 $((total_validators-1))); do
  snarkos clean "--dev=$validator_index" "--network=$network_id"
  if (( validator_index == outdated_validator )); then
    heights=$outdated_heights
  else
    heights=$upgraded_heights
  fi
  CONSENSUS_VERSION_HEIGHTS=$heights run_with_prefix "validator-$validator_index" snarkos start "${common_flags[@]}" \
    "--dev=$validator_index" --validator "--logfile=$log_dir/validator-$validator_index.log" \
    "--rest=127.0.0.1:$((3030+validator_index))" --no-dev-txs
  PIDS[validator_index]=$!
  sleep 1
done

wait_for_nodes "$total_validators" 0 "$network_name" 180

# Wait for the upgraded validators to pass the activation height.
if ! wait_for_heights 0 "$outdated_validator" "$target_height" "$network_name" 1800 5; then
  log "⛔️ Upgraded validators did not reach height $target_height"
  exit 1
fi
log "Upgraded validators reached height $target_height"

port=$((3030+outdated_validator))
log_file="$log_dir/validator-$outdated_validator.log"

# Every validator, including the outdated one, is still running.
if check_node_stopped; then
  exit 1
fi

# The outdated validator stopped before the activation height.
height=$(get_block_height_by_port "$port" "$network_name" 5)
log "Outdated validator height: $height"
if [[ -z "$height" ]] || (( height >= activation_height )); then
  log "⛔️ The outdated validator reached height '$height'"
  exit 1
fi

# The outdated validator reports that it is not synced.
sync_status=$(curl -s --max-time 5 "http://127.0.0.1:$port/v2/$network_name/sync/status")
log "Outdated validator sync status: $sync_status"
if ! echo "$sync_status" | grep -q '"is_synced": false'; then
  log "⛔️ The outdated validator reports that it is synced"
  exit 1
fi

# The outdated validator keeps logging that it must be upgraded.
count=$(grep -c "must be upgraded" "$log_file" || true)
log "Outdated validator logged the upgrade error $count times"
if (( count < 2 )); then
  log "⛔️ The outdated validator did not repeatedly log the upgrade error"
  exit 1
fi

log "✅ The outdated validator went dark and the network continued"
