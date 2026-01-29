#!/usr/bin/env bash
set -euo pipefail

# ==========================================
# CONFIGURATION
# ==========================================

# Argument 1: one or more port ranges (comma-separated)
#   Example: "5000-5003"
#   Example: "5000-5003,6000-6005"
TARGET_RANGES_RAW="${1:-}"

if [[ -z "$TARGET_RANGES_RAW" ]]; then
  echo "Error: No port range(s) provided."
  echo "Usage: ./chaos-runner.sh <PORT-RANGE[,PORT-RANGE...]> <COMMAND> [ARGS...]"
  echo "Example: ./chaos-runner.sh 5000-5003 ./devnet.sh"
  echo "Example: ./chaos-runner.sh 5000-5003,6000-6005 ./devnet.sh --flag"
  exit 1
fi

shift

if [[ $# -eq 0 ]]; then
  echo "Error: No test command provided."
  exit 1
fi

# Chaos Timing (in seconds)
MIN_WAIT=10
MAX_WAIT=30
MIN_DURATION=15
MAX_DURATION=30

# Presets from delay-network.sh (order matters for mapping)
PRESETS=("spikes") # add more if needed: ("spikes" "bad-wifi" ...)
NETWORK_SCRIPT="./scripts/delay-network.sh"

# ==========================================
# PARSING + MAPPING LOGIC
# ==========================================

# Split ranges by comma into array TARGET_RANGES
IFS=',' read -r -a TARGET_RANGES <<< "$TARGET_RANGES_RAW"

# Trim whitespace around each range (basic)
for i in "${!TARGET_RANGES[@]}"; do
  TARGET_RANGES[$i]="${TARGET_RANGES[$i]#"${TARGET_RANGES[$i]%%[![:space:]]*}"}"
  TARGET_RANGES[$i]="${TARGET_RANGES[$i]%"${TARGET_RANGES[$i]##*[![:space:]]}"}"
done

NUM_RANGES="${#TARGET_RANGES[@]}"
NUM_PRESETS="${#PRESETS[@]}"

if [[ "$NUM_RANGES" -lt 1 ]]; then
  echo "Error: No valid ranges parsed from '$TARGET_RANGES_RAW'."
  exit 1
fi

if [[ "$NUM_PRESETS" -lt 1 ]]; then
  echo "Error: PRESETS array is empty."
  exit 1
fi

# We build RANGE_PRESET_IDX_LISTS so that each range gets a list of preset indices.
# Each chaos cycle:
#   - for each range, pick 1 preset randomly from its assigned list and apply it to that range
#
# Mapping rules (generalized from your examples):
# - If presets >= ranges:
#     ranges 1..(N-1) get presets 1..(N-1) respectively
#     range N gets presets N..M (all remaining)
# - If presets < ranges:
#     ranges 1..(M-1) get presets 1..(M-1) respectively
#     ranges M..N get preset M (the last preset)
#
# Special case:
# - If M == 1: all ranges get preset 1

declare -a RANGE_PRESET_IDX_LISTS=()  # each entry: "0 1 2" etc.

if [[ "$NUM_PRESETS" -ge "$NUM_RANGES" ]]; then
  # First N-1 ranges get one preset each.
  for ((r=0; r<NUM_RANGES; r++)); do
    if [[ "$r" -lt $((NUM_RANGES - 1)) ]]; then
      RANGE_PRESET_IDX_LISTS[$r]="$r"
    else
      # Last range gets the remaining presets r..(M-1)
      idxs=()
      for ((p=r; p<NUM_PRESETS; p++)); do
        idxs+=("$p")
      done
      RANGE_PRESET_IDX_LISTS[$r]="${idxs[*]}"
    fi
  done
else
  # presets < ranges
  # First M-1 ranges get one preset each; remaining ranges get last preset (M-1)
  last_preset_idx=$((NUM_PRESETS - 1))
  for ((r=0; r<NUM_RANGES; r++)); do
    if [[ "$r" -lt $((NUM_PRESETS - 1)) ]]; then
      RANGE_PRESET_IDX_LISTS[$r]="$r"
    else
      RANGE_PRESET_IDX_LISTS[$r]="$last_preset_idx"
    fi
  done
fi

print_mapping() {
  echo "[Chaos Runner] Range->Preset mapping (per cycle, 1 preset chosen randomly from assigned list):"
  for ((r=0; r<NUM_RANGES; r++)); do
    range="${TARGET_RANGES[$r]}"
    idx_list="${RANGE_PRESET_IDX_LISTS[$r]}"
    names=()
    for idx in $idx_list; do
      names+=("${PRESETS[$idx]}")
    done
    echo "  - Range $((r+1)) ($range) <= ${names[*]}"
  done
}

# ==========================================
# CHAOS LOGIC
# ==========================================

reset_network() {
  echo "[Chaos Runner] Clearing network rules..."
  sudo "$NETWORK_SCRIPT" clear >/dev/null
}

cleanup() {
  # Best-effort cleanup
  reset_network || true
  if [[ -n "${CHAOS_PID:-}" ]]; then
    kill "$CHAOS_PID" 2>/dev/null || true
    wait "$CHAOS_PID" 2>/dev/null || true
  fi
}

trap cleanup EXIT

pick_random_from_list() {
  # Args: list of integers (space-separated)
  local list=($1)
  local count="${#list[@]}"
  if [[ "$count" -eq 1 ]]; then
    echo "${list[0]}"
    return 0
  fi
  echo "${list[$((RANDOM % count))]}"
}

apply_chaos_once() {
  # Apply one chaos preset per range, using the mapping lists.
  for ((r=0; r<NUM_RANGES; r++)); do
    range="${TARGET_RANGES[$r]}"
    idx_list="${RANGE_PRESET_IDX_LISTS[$r]}"
    chosen_idx="$(pick_random_from_list "$idx_list")"
    chosen_preset="${PRESETS[$chosen_idx]}"

    echo "[Chaos Runner] Applying '$chosen_preset' on range '$range'..."
    sudo "$NETWORK_SCRIPT" "$chosen_preset" "$range"
  done
}

chaos_loop() {
  echo "[Chaos Runner] Background chaos loop started on ranges: ${TARGET_RANGES[*]}"
  print_mapping

  while true; do
    # 1. Stay Clean
    WAIT_TIME="$(shuf -i "$MIN_WAIT-$MAX_WAIT" -n 1)"
    echo "[Chaos Runner] Network healthy for ${WAIT_TIME}s..."
    sleep "$WAIT_TIME"

    # 2. Apply Chaos
    CHAOS_DURATION="$(shuf -i "$MIN_DURATION-$MAX_DURATION" -n 1)"
    echo "[Chaos Runner] Applying chaos for ${CHAOS_DURATION}s..."
    apply_chaos_once

    sleep "$CHAOS_DURATION"

    # 3. Reset
    reset_network
  done
}

# ==========================================
# MAIN EXECUTION
# ==========================================

chaos_loop &
CHAOS_PID=$!

echo "[Chaos Runner] Starting main test command: $*"
"$@"
EXIT_CODE=$?

echo "[Chaos Runner] Main command finished with exit code $EXIT_CODE."
exit "$EXIT_CODE"
