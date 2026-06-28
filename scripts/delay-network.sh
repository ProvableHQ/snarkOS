#!/usr/bin/env bash

# Set strict mode and IFS to catch unset vars, pipeline failures and whitespace issues.
set -euo pipefail
IFS=$'\n\t'

# Configuration, can be overriden if necessary:
INTERFACE="${DELAY_NETWORK_INTERFACE:-lo}"
PARENT="${DELAY_NETWORK_PARENT:-1:3}"
HANDLE="${DELAY_NETWORK_HANDLE:-30:}"

# One function to print the error and exit when needed.
function die() {
  echo "Error: $*" >&2
  exit 1
}

# Check if the a command this script depends on exist
function need_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "Missing required command: $1"
}

# Print the command before executing
function run() {
    # We use a subshell ( ... ) to temporarily set IFS to space for the echo command only.
    # This ensures "  + tc qdisc..." prints on one line despite the global IFS setting.
    ( IFS=' '; echo "  + $*" )
    "$@"
}

function usage() {
    cat <<EOF
Usage: sudo ./delay-network.sh [COMMAND] [PORTS] [OPTIONS]

Description:
  A wrapper for Linux Traffic Control (tc) to simulate network conditions
  on specific ports of the loopback interface.

Commands:
  clear               Remove all delays and reset to normal.
  status              Show current active rules and monitored ports.

Presets (Simplest Usage):
  lan [PORTS]         Congested LAN:    5ms  ± 2ms  (Normal dist)
  wan [PORTS]         Trans-Oceanic:    200ms ± 50ms (Normal dist)
  spikes [PORTS]      Lag Spikes:       20ms ± 800ms (Pareto dist)
  bad-wifi [PORTS]    Packet Loss:      50ms ± 30ms + 2% Loss

Custom Configuration:
  custom [PORTS] [DELAY] [JITTER] [DIST] [LOSS]

  This command allows you to build specific network profiles.

  Arguments:
    PORTS       Single port (8080) or range (8000-8010).
    DELAY       Base latency (e.g., 100ms).
    JITTER      Variance, which can include correlation.
                Format: "TIME" or "TIME CORRELATION%"
                Example: "50ms" or "50ms 25%"
    DIST        Distribution curve: 'uniform', 'normal', 'pareto', or 'paretonormal'.
                Use 'uniform' (or leave empty) for standard random spread.
    LOSS        (Optional) Packet loss percentage (e.g., 1%, 0.5%).

Examples:
  1. Simple Uniform Delay (100ms to 200ms):
     sudo ./delay-network.sh custom 8080 150ms 50ms uniform

  2. "Real" Internet (Correlation + Normal Dist):
     sudo ./delay-network.sh custom 8000-8005 100ms "20ms 25%" normal

  3. Extreme Stress (Pareto Spikes + Loss):
     sudo ./delay-network.sh custom 8080 20ms "1000ms 25%" pareto 5%
EOF
    exit 1
}

function reset_tc() {
    echo "Cleaning up existing rules (if any)..."
    tc qdisc del dev "$INTERFACE" root 2> /dev/null || true
}

function setup_root() {
    # Create the Priority Queue
    # Band 1 & 2: Default
    # Band 3: Delayed
    run tc qdisc add dev "$INTERFACE" root handle 1: prio
}

function apply_filter() {
    local ports="$1"
    local start_port
    local end_port

    if [[ "$ports" == *"-"* ]]; then
        start_port="${ports%-*}" # Remove the suffix starting with '-'
        end_port="${ports#*-}" # Remove the prefix ending on '-'
    else
        start_port="$ports"
        end_port="$ports"
    fi

    # Validate numeric
    [[ "$start_port" =~ ^[0-9]+$ ]] || die "Invalid port: '$start_port'"
    [[ "$end_port" =~ ^[0-9]+$ ]] || die "Invalid port: '$end_port'"

    # Validate range
    (( start_port >= 1 && start_port <= 65535 )) || die "Port out of range: $start_port"
    (( end_port >= 1 && end_port <= 65535 )) || die "Port out of range: $end_port"
    (( start_port <= end_port )) || die "Invalid port range: $start_port-$end_port (start > end)"

    echo "Applying filters for ports $start_port to $end_port..."

    for port in $(seq "$start_port" "$end_port"); do
        # Filter for IPv4
        run tc filter add dev "$INTERFACE" protocol ip parent 1:0 prio 1 u32 \
            match ip dport "$port" 0xffff flowid "$PARENT" > /dev/null

        # Filter for IPv6
        run tc filter add dev "$INTERFACE" protocol ipv6 parent 1:0 prio 2 u32 \
            match ip6 dport "$port" 0xffff flowid "$PARENT" > /dev/null
    done
}

function apply_netem() {
    local delay="$1"
    local jitter="${2:-}"
    local dist="${3:-}"
    local loss="${4:-}"

    # Build the command as an array (no eval), and split jitter safely (may be "800ms 25%")
    local -a cmd=(tc qdisc add dev "$INTERFACE" parent "$PARENT" handle "$HANDLE" netem delay "$delay")

    if [[ -n "${jitter:-}" ]]; then
        # Split jitter into 1 or 2 args:
        #   "50ms"        -> ("50ms")
        #   "50ms 25%"    -> ("50ms" "25%")
        local jitter_a jitter_b

        jitter_a="${jitter%% *}"
        if [[ "$jitter" == *" "* ]]; then
            jitter_b="${jitter#* }"
            cmd+=("$jitter_a" "$jitter_b")
        else
            cmd+=("$jitter_a")
        fi
    fi

    # Add Distribution
    if [[ -n "${dist:-}" && "$dist" != "uniform" ]]; then
        cmd+=(distribution "$dist")
    fi

    # Add Packet Loss
    if [[ -n "${loss:-}" ]]; then
        cmd+=(loss "$loss")
    fi

    echo "Executing Kernel Command:"
    # We delegate printing entirely to 'run' to avoid formatting issues
    run "${cmd[@]}"
}

# --- Main Execution ---

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" || "${1:-}" == "help" ]]; then
    usage
fi

if [ $# -lt 1 ]; then
    usage
fi

need_cmd tc

COMMAND=$1
PORTS=${2:-}

if [ "$EUID" -ne 0 ]; then 
  echo "Error: This script requires root privileges. Try 'sudo ./delay-network.sh ...'"
  exit 1
fi

case "$COMMAND" in
    clear)
        reset_tc
        echo "Network reset to normal."
        ;;

    status)
        echo "--- Active Qdiscs (Delays) ---"
        tc qdisc show dev "$INTERFACE" | grep "netem" || echo "(none)" # To not terminate the script because of bad grep

        echo ""
        echo "--- Monitored Ports (Filters) ---"
        # Logic: 
        # 1. List filters
        # 2. Find lines with "match" (e.g., "match 00001388/0000ffff at 20")
        # 3. Extract 2nd column (00001388/0000ffff)
        # 4. Cut before the slash to get hex value (00001388)
        ( tc filter show dev "$INTERFACE" | grep "match" || true ) \
          | awk '{print $2}' | cut -d'/' -f1 | while read -r hex; do
            if [[ -n "$hex" ]]; then
              printf "%d\n" "0x$hex"
            fi
          done | sort -u
        ;;

    lan)
        [[ -n "$PORTS" ]] || die "Missing PORTS for 'lan'. Example: sudo ./delay-network.sh lan 8080"

        reset_tc
        setup_root
        apply_netem "5ms" "2ms" "normal"
        apply_filter "$PORTS"
        ;;

    wan)
        [[ -n "$PORTS" ]] || die "Missing PORTS for 'wan'. Example: sudo ./delay-network.sh wan 8000-8005"

        reset_tc
        setup_root
        apply_netem "200ms" "50ms" "normal"
        apply_filter "$PORTS"
        ;;

    spikes)
        [[ -n "$PORTS" ]] || die "Missing PORTS for 'spikes'. Example: sudo ./delay-network.sh spikes 8080"

        reset_tc
        setup_root
        apply_netem "20ms" "800ms 25%" "pareto"
        apply_filter "$PORTS"
        ;;

    bad-wifi)
        [[ -n "$PORTS" ]] || die "Missing PORTS for 'bad-wifi'. Example: sudo ./delay-network.sh bad-wifi 8080"

        reset_tc
        setup_root
        apply_netem "50ms" "30ms 25%" "normal" "2%"
        apply_filter "$PORTS"
        ;;

    custom)
        if [ $# -lt 3 ]; then
            die "Missing arguments for custom mode. Example: sudo ./delay-network.sh custom PORTS DELAY [JITTER] [DIST] [LOSS]"
        fi
        [[ -n "$PORTS" ]] || die "Missing PORTS for 'custom'. Example: sudo ./delay-network.sh custom 8080 100ms \"20ms 25%\" normal"

        reset_tc
        setup_root
        # Args: delay jitter distribution loss
        apply_netem "$3" "${4:-}" "${5:-}" "${6:-}"
        apply_filter "$PORTS"
        ;;

    *)
        usage
        ;;
esac
