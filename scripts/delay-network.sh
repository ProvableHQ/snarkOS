#!/bin/bash

# Configuration
INTERFACE="lo"
PARENT="1:3"
HANDLE="30:"

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
    echo "Cleaning up existing rules..."
    tc qdisc del dev $INTERFACE root 2> /dev/null
}

function setup_root() {
    # Create the Priority Queue
    # Band 1 & 2: Default
    # Band 3: Delayed
    tc qdisc add dev $INTERFACE root handle 1: prio
}

function apply_filter() {
    local ports=$1
    local start_port
    local end_port

    if [[ $ports == *"-"* ]]; then
        start_port=$(echo $ports | cut -d'-' -f1)
        end_port=$(echo $ports | cut -d'-' -f2)
    else
        start_port=$ports
        end_port=$ports
    fi

    echo "Applying filters for ports $start_port to $end_port..."

    for (( port=$start_port; port<=$end_port; port++ )); do
        # Filter for IPv4
        tc filter add dev $INTERFACE protocol ip parent 1:0 prio 1 u32 \
            match ip dport $port 0xffff flowid $PARENT
        
        # Filter for IPv6
        tc filter add dev $INTERFACE protocol ipv6 parent 1:0 prio 2 u32 \
            match ip6 dport $port 0xffff flowid $PARENT
    done
}

function apply_netem() {
    local delay=$1
    local jitter=$2
    local dist=$3
    local loss=$4
    
    # Base command
    CMD="tc qdisc add dev $INTERFACE parent $PARENT handle $HANDLE netem delay $delay $jitter"
    
    # Add Distribution (only if not uniform, as uniform is default/implicit)
    if [ ! -z "$dist" ] && [ "$dist" != "uniform" ]; then
        CMD="$CMD distribution $dist"
    fi

    # Add Packet Loss
    if [ ! -z "$loss" ]; then
        CMD="$CMD loss $loss"
    fi

    echo "Executing Kernel Command:"
    echo "  $CMD"
    eval $CMD
}

# --- Main Execution ---

if [ $# -lt 1 ]; then
    usage
fi

COMMAND=$1
PORTS=$2

if [ "$EUID" -ne 0 ]; then 
  echo "Error: This script requires root privileges. Try 'sudo ./delay-network.sh ...'"
  exit 1
fi

case $COMMAND in
    clear)
        reset_tc
        echo "Network reset to normal."
        ;;
        
    status)
        echo "--- Active Qdiscs (Delays) ---"
        tc qdisc show dev $INTERFACE | grep "netem"
    
        echo ""
        echo "--- Monitored Ports (Filters) ---"
        # Logic: 
        # 1. List filters
        # 2. Find lines with "match" (e.g., "match 00001388/0000ffff at 20")
        # 3. Extract 2nd column (00001388/0000ffff)
        # 4. Cut before the slash to get hex value (00001388)
        tc filter show dev $INTERFACE | grep "match" | awk '{print $2}' | cut -d'/' -f1 | while read hex; do
            # 0x$hex tells printf to interpret it as a hexadecimal number
            if [ ! -z "$hex" ]; then
                printf "%d\n" "0x$hex"
            fi
        done | sort -u
        ;;

    lan)
        reset_tc
        setup_root
        apply_netem "5ms" "2ms" "normal"
        apply_filter $PORTS
        ;;

    wan)
        reset_tc
        setup_root
        apply_netem "200ms" "50ms" "normal"
        apply_filter $PORTS
        ;;

    spikes)
        reset_tc
        setup_root
        apply_netem "20ms" "800ms 25%" "pareto"
        apply_filter $PORTS
        ;;

    bad-wifi)
        reset_tc
        setup_root
        apply_netem "50ms" "30ms 25%" "normal" "2%"
        apply_filter $PORTS
        ;;

    custom)
        if [ $# -lt 4 ]; then
            echo "Error: Missing arguments for custom mode."
            echo "Run without arguments to see detailed usage."
            exit 1
        fi
        reset_tc
        setup_root
        # Args: delay jitter distribution loss
        apply_netem "$3" "$4" "$5" "$6"
        apply_filter $PORTS
        ;;

    *)
        usage
        ;;
esac
