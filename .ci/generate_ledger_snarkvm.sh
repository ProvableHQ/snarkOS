#!/bin/bash

###########################################################
# Generates a ledger using the testchain-generator tool
# from snarkVM.
# For this to work correctly you need to install the binary
# from a snarkvm version comptible (ideally identical) to
# this commit of snarkOS.
###########################################################

set -eo pipefail # error on any command failure

# Uncomment this to print commands before executing them for easier debugging.
#set -x

# Set parameters directly
num_validators=$1
num_blocks=$2
network_id=1

# Default values if not provided
: "${total_validators:=40}"
: "${min_height:=250}"

. ./.ci/utils.sh

git_commit=$(git rev-parse --short=10 HEAD)
echo "On git commit ${git_commit}"

printf "num_validators=${num_validators}, git_commit=${git_commit}, snapshot_height=${num_blocks}" > info.txt

snarkvm-testchain-generator "$num_validators" "$num_blocks"

zipname="sync-ledger-val${num_validators}-${min_height}-${git_commit}.zip"
echo "Done! Generating zipfile \"$zipname\""
zip "$zipname" ".ledger-${network_id}-0" info.txt
