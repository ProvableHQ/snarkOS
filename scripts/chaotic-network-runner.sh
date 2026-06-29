#!/usr/bin/env bash

# ==========================================
# CONFIGURATION
# ==========================================

# 1. Require Port Range (Argument 1)
TARGET_PORTS=$1

if [ -z "$TARGET_PORTS" ]; then
    echo "Error: No port range provided."
    echo "Usage: ./chaos-runner.sh <PORT-RANGE> <COMMAND>"
    echo "Example: ./chaotic-network-runner.sh 5000-5003 ./devnet.sh"
    exit 1
fi

# Shift arguments so "$@" now only contains the command to run
shift

# Check if there is a command to run left
if [ $# -eq 0 ]; then
    echo "Error: No test command provided."
    exit 1
fi

# Chaos Timing (in seconds)
MIN_WAIT=10
MAX_WAIT=30
MIN_DURATION=15
MAX_DURATION=30

# Presets from delay-network.sh
PRESETS=("lan" "wan" "spikes") # "bad-wifi" might be too volatile for a generic test
NETWORK_SCRIPT="./scripts/delay-network.sh"

# ==========================================
# CHAOS LOGIC
# ==========================================

reset_network() {
    echo "[Chaos Runner] 🧹 Clearing network rules..."
    sudo $NETWORK_SCRIPT clear > /dev/null
}

trap reset_network EXIT

chaos_loop() {
    echo "[Chaotic Network Runner] 🎲 Background chaos loop started on ports $TARGET_PORTS."

    while true; do
        # 1. Stay Clean
        WAIT_TIME=$(shuf -i $MIN_WAIT-$MAX_WAIT -n 1)
        echo "[Chaos Runner] 🟢 Network healthy for ${WAIT_TIME}s..."
        sleep "$WAIT_TIME"

        # 2. Select Random Preset
        RANDOM_PRESET=${PRESETS[$RANDOM % "${#PRESETS[@]}"]}

        # 3. Apply Chaos
        CHAOS_DURATION=$(shuf -i $MIN_DURATION-$MAX_DURATION -n 1)
        echo "[Chaos Runner] 💥 Applying '$RANDOM_PRESET' on ports $TARGET_PORTS for ${CHAOS_DURATION}s..."

        sudo $NETWORK_SCRIPT "$RANDOM_PRESET" "$TARGET_PORTS"

        sleep "$CHAOS_DURATION"

        # 4. Reset
        reset_network
    done
}

# ==========================================
# MAIN EXECUTION
# ==========================================

# Start chaos in background
chaos_loop &
CHAOS_PID=$!

# Run the actual test command (passed as arguments)
echo "[Chaos Runner] 🚀 Starting main test command: $*"
"$@"
EXIT_CODE=$?

# Cleanup
echo "[Chaos Runner] 🏁 Main command finished with exit code $EXIT_CODE."
kill $CHAOS_PID 2>/dev/null
wait $CHAOS_PID 2>/dev/null

exit $EXIT_CODE
