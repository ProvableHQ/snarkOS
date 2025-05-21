#!/bin/bash
# USAGE examples: 
  # CLI with env vars: PROVER_PRIVATE_KEY=APrivateKey1...  ./run-prover.sh
  # CLI with prompts for vars:  ./run-prover.sh
  # CLI with CUDA enabled ./run-prover.sh --cuda

# If the env var PROVER_PRIVATE_KEY is not set, prompt for it
if [ -z "${PROVER_PRIVATE_KEY}" ]
then
  read -r -p "Enter the Aleo Prover account private key: "
  PROVER_PRIVATE_KEY=$REPLY
fi

if [ "${PROVER_PRIVATE_KEY}" == "" ]
then
  echo "Missing account private key. (run 'snarkos account new' and try again)"
  exit
fi

for word in "$@"; do
  if [ "$word" == "--cuda" ]; then
    ENABLE_CUDA=true
  else
    ARGS+=("$word")
  fi
done

# Build the command with optional CUDA feature
if [ "$ENABLE_CUDA" == "true" ]; then
  COMMAND="cargo run --release --features cuda -- start --nodisplay --prover --private-key ${PROVER_PRIVATE_KEY}"
else
  COMMAND="cargo run --release -- start --nodisplay --prover --private-key ${PROVER_PRIVATE_KEY}"
fi

# Append other arguments (excluding --cuda)
for arg in "${ARGS[@]}"; do
  COMMAND="${COMMAND} ${arg}"
done

function exit_node()
{
    echo "Exiting..."
    kill $!
    exit
}

trap exit_node SIGINT

echo "Checking for updates..."
git stash
STATUS=$(git pull)

if [ "$STATUS" != "Already up to date." ]; then
  echo "Updated code found, cleaning the project"
  cargo clean
fi

echo "Running an Aleo Prover node..."
$COMMAND &
wait
