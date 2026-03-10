#!/bin/bash

####################################################
# Measures a client syncing 1000 blocks from the CDN
####################################################

set -eo pipefail # error on any command failure

network_id=0
min_height=250

# Adjust this to show more/less log messages
log_filter="info,snarkos_node_rest=warn,snarkos_node_cdn=debug"

max_wait=1800 # Wait for up to 30 minutes
poll_interval=1 # Check block heights every second

# shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh

# Create log directory
init_log_dir

network_name=$(get_network_name $network_id)
log "Using network: $network_name (ID: $network_id)"

# Define a trap handler that cleans up all processes on exit.
# shellcheck disable=SC2329
function exit_handler() {
  stop_nodes
}
trap exit_handler EXIT

# Define a trap handler that prints a message when an error occurs.
trap 'log "⛔️ Error in $BASH_SOURCE at line $LINENO: \"$BASH_COMMAND\" failed (exit $?)"' ERR

# Ensure there are no old ledger files and the node syncs from scratch
snarkos clean "--network=$network_id" || true

# Arguments to pass to the node
args=(
  "--network=$network_id"
  --nobanner --noupdater --nodisplay # reduce clutter in the output and hide TUI
  --rest-rps=1000000 # ensure benchmarks don't fail due to rate limiting
  "--log-filter=$log_filter" # only show the logs we care about
)

# Spawn the client that will sync the ledger.
# Use the same CPU cores as in the other benchmarks, so the numbers are comparable.
# shellcheck disable=SC2086
run_with_prefix "client-0" $TASKSET2 snarkos start --client "${args[@]}"
PIDS[0]=$!

wait_for_nodes 0 1 "$network_name"

# Check heights periodically with a timeout
start=$(now)
while (( $(elapsed_since "$start") < max_wait )); do
  total_wait=$(elapsed_since "$start")
  if check_heights 0 1 $min_height "$network_name" "$total_wait"; then
    throughput=$(compute_throughput "$min_height" "$total_wait")

    log "🎉 Benchmark done! Waited ${total_wait}s for $min_height blocks. Throughput was $throughput blocks/s."

    # Append data to results file.
    printf "{ \"name\": \"cdn-sync\", \"unit\": \"blocks/s\", \"value\": %.3f, \"extra\": \"total_wait=%is, target_height=${min_height}\" }\n" \
       "$throughput" "$total_wait" | tee -a results.json
    exit 0
  fi
  
  # Continue waiting
  sleep "$poll_interval"
done

log "❌ Benchmark failed! Client did not sync within 30 minutes."

exit 1
