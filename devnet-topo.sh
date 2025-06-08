#!/bin/bash

# devnet-topo.sh

# Like devnet.sh, but with these changes that stabilize the network topology:
#
# * Only asks for the number N of validators, not the number of clients.
#   Makes the same number of clients as validators, to reflect
#   the concept that each validator has has a dedicated core client.
#   (Currently we are waiting for more information about how mainnet validators
#   set up their trusted peers; this script can then be updated to match.)
# * For the Gateway (communication for BFT consensus)
#   sets --validators starting up each validator to all the other validators
#   (devnet.sh ends up setting it only to the first two validators).
# * For the Router (p2p communication), sets --peers (the trusted peers)
#   directly, rather than allowing them to be set by parse_development(),
#   * Each validator has one trusted peer: its core client.
#     No longer allow validators to accept external peer connections
#     (removed the "--allow-external-peers" flag).
#   * Each client has N trusted peers: its one trusted validator
#     and the N-1 other clients.

if [[ -n "$TMUX" ]]; then
   echo "Detected nested tmux session. Try again after unsetting \$TMUX, e.g., using \`unset TMUX\` in bash."
   exit 1
fi

# Read the total number of validators from the user or use a default value of 4
read -p "Enter the total number of validators (default: 4): " total_validators
total_validators=${total_validators:-4}

# Each validator trusts one core client.
total_clients=${total_validators}

# Read the network ID from user or use a default value of 1
read -p "Enter the network ID (mainnet = 0, testnet = 1, canary = 2) (default: 1): " network_id
network_id=${network_id:-1}

# Ask the user if they want to run 'cargo install --locked --path .' or use a pre-installed binary
read -p "Do you want to run 'cargo install --locked --path .' to build the binary? (y/n, default: y): " build_binary
build_binary=${build_binary:-y}

# Ask the user whether to clear the existing ledger history
read -p "Do you want to clear the existing ledger history? (y/n, default: n): " clear_ledger
clear_ledger=${clear_ledger:-n}

# Log verbosity is set to 1 (DEBUG) by default.
verbosity=1

if [[ $build_binary == "y" ]]; then
  # Ask the user if they want to enable validator telemetry
  read -p "Do you want to enable validator telemetry? (y/n, default: y): " enable_telemetry
  enable_telemetry=${enable_telemetry:-y}

  # Build command
  build_cmd="cargo install --locked --path ."

  # Add the telemetry feature if requested
  if [[ $enable_telemetry == "y" ]]; then
    build_cmd+=" --features telemetry"
  fi

  # Build command
  echo "Running build command: \"$build_cmd\""
  eval "$build_cmd" || exit 1
fi

# Clear the ledger logs for each validator if the user chooses to clear ledger
if [[ $clear_ledger == "y" ]]; then
  # Create an array to store background processes
  clean_processes=()

  for ((index = 0; index < $((total_validators + total_clients)); index++)); do
    # Run 'snarkos clean' for each node in the background
    snarkos clean --network $network_id --dev $index &

    # Store the process ID of the background task
    clean_processes+=($!)
  done

  # Wait for all 'snarkos clean' processes to finish
  for process_id in "${clean_processes[@]}"; do
    wait "$process_id"
  done
fi

# Create a timestamp-based directory for log files
log_dir=".logs-$(date +"%Y%m%d%H%M%S")"
mkdir -p "$log_dir"

# Create a new tmux session named "devnet"
tmux new-session -d -s "devnet" -n "validator-0"
if [[ $? -ne 0 ]]; then
   echo "Failed to create new TMUX session."
   exit 1
fi

# Get the tmux's base-index for windows
# we have to create all windows with index offset by this much
index_offset="$(tmux show-option -gv base-index)"
if [ -z "$index_offset" ]; then
  index_offset=0
fi

# Generate validator indices from 0 to (total_validators - 1).
validator_indices=($(seq 0 $((total_validators - 1))))

# Loop through the list of validator indices and create a new window for each
for validator_index in "${validator_indices[@]}"; do
  # Generate a unique and incrementing log file name based on the validator index
  name="validator-$validator_index"
  log_file="$log_dir/$name.log"
  window_index=$((validator_index + index_offset))
  metrics_flag=""

  if [ "$validator_index" -eq 0 ]; then
    # Enable metrics only for the first validator.
    metrics_flag="--metrics"
    # We don't need to create a window for the first validator because the tmux
    # session already starts with one window.
  else
    # Create a new window with a unique name
    tmux new-window -t "devnet:$window_index" -n $name
  fi

  # Create the list of trusted validators for the validator at validator_index
  trusted_validators=""
  for trusted_validator_index in "${validator_indices[@]}"; do
      if [ "$validator_index" -ne "$trusted_validator_index" ]; then
          if [ "$trusted_validators" = "" ]; then
              trusted_validators="127.0.0.1:$((5000 + trusted_validator_index))"
          else
              trusted_validators="${trusted_validators},127.0.0.1:$((5000 + trusted_validator_index))"
          fi
      fi
  done

  # Create the list of peers (i.e., trusted peers) for the validator at the_validator_index
  # It is the corresponding client now.
  # I.e., validators 0,1,2,3 each has one trusted peer which is the client at ports 4130 + num_validators + 0,1,2,3, respectively
  validator_trusted_peers="127.0.0.1:$((4130 + total_validators + validator_index))";

  # Send the command to start the validator to the new window and capture output to the log file
  tmux send-keys -t "devnet:$window_index" "snarkos start --nodisplay --network $network_id --dev $validator_index --dev-num-validators $total_validators --validator --validators $trusted_validators --peers $validator_trusted_peers --logfile $log_file --verbosity $verbosity $metrics_flag" C-m
done

# Define a function that generates the string to be passed as --peers when starting a client node.
# The trusted peers consist of the one paired validator plus all the other clients.
# The string looks like "vvv${i},ccc0,ccc1,...,ccc${n}"
# excluding ccc${i} from the list,
# where vvv${i} is "127.0.0.1:$((4130 + i))"
# and ccc${i} is "127.0.0.1:$((4130 + total_validators + i))"
client_trusted_peers() {
    # n is the number of validators (= number of clients)
    # i is the index of the client for which we want a trusted peers list
    local n=$1
    local i=$2

    local result="127.0.0.1:$((4130 + i))"

    # Appends all ccc{number} from 0 to n-1, except ccc${i}
    for ((j=0; j<n; j++)); do
        if [ $j -ne $i ]; then
            result="${result},127.0.0.1:$((4130 + total_validators + j))"
        fi
    done

    echo "$result"
}

if [ "$total_clients" -ne 0 ]; then
  # Generate client indices from 0 to (total_clients - 1)
  client_indices=($(seq 0 $((total_clients - 1))))

  # Loop through the list of client indices and create a new window for each
  for client_index in "${client_indices[@]}"; do
    # Generate a unique and incrementing log file name based on the client index
    name="client-$client_index"
    log_file="$log_dir/$name.log"

    window_index=$((client_index + total_validators + index_offset))

    # Create a new window with a unique name
    tmux new-window -t "devnet:$window_index" -n $name

    # Send the command to start the client to the new window and capture output to the log file
    tmux send-keys -t "devnet:$window_index" "snarkos start --nodisplay --network $network_id --dev $((total_validators + client_index)) --dev-num-validators $total_validators --client --peers $(client_trusted_peers $total_validators $client_index) --logfile $log_file  --verbosity $verbosity" C-m
  done
fi

# Attach to the tmux session to view and interact with the windows
tmux attach-session -t "devnet"
