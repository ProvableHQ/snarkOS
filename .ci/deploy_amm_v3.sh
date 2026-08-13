#!/bin/bash
#
# deploy_amm_v3.sh
#   Variant of ../amm-v3/deploy_local.sh for snarkOS E2E tests.
#
#   Deploys all local amm-v3 programs to a running snarkOS devnet after the
#   latest consensus version is active. Idempotent: programs already on-chain
#   are skipped.
#
# Usage:
#   ./.ci/deploy_amm_v3.sh                 # build + deploy everything
#   ./.ci/deploy_amm_v3.sh --amm-only      # rebuild + redeploy just the AMM
#   ./.ci/deploy_amm_v3.sh --dry-run       # print steps, don't broadcast
#   ./.ci/deploy_amm_v3.sh --dev-key 0     # deploy with snarkOS development key 0 (default)
#
# Environment:
#   AMM_V3_ROOT   Path to the amm-v3 repo (default: ../amm-v3 relative to snarkOS)
#   NETWORK_ID    0=mainnet, 1=testnet, 2=canary (default: 0)
#   ENDPOINT      snarkOS REST base URL (default: http://127.0.0.1:3030)
#   DEV_KEY       snarkOS development key index (default: 0)
#
# --dev-key 0:
#   private_key: APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH
#   address:     aleo1rhgdu77hgyqd3xjj8ucu3jj9r2krwz6mnzyd80gncr5fxcwlh5rsvzp9px
#
# --dev-key 1:
#   private_key: APrivateKey1zkp2RWGDcde3efb89rjhME1VYA8QMxcxep5DShNBR6n8Yjh
#   address:     aleo1s3ws5tra87fjycnjrwsjcrnw2qxr8jfqqdugnf0xzqqw29q9m5pqem2u4t
#
# Deployments use `leo deploy --devnet` (dummy proofs) because the AMM is too
# large for real SNARK generation in CI. Post-deploy initialization uses
# `snarkos developer execute --dev-key`.

set -euo pipefail

# shellcheck source=SCRIPTDIR/utils.sh
. ./.ci/utils.sh
mkdir -p "$log_dir"

AMM_ONLY=0
DRY_RUN=0
DEV_KEY="${DEV_KEY:-0}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --amm-only) AMM_ONLY=1 ;;
    --dry-run)  DRY_RUN=1  ;;
    --dev-key)
      if [[ $# -lt 2 ]]; then
        echo "--dev-key requires an index (0 or 1)" >&2
        exit 2
      fi
      DEV_KEY="$2"
      shift
      ;;
    -h|--help)
      sed -n '2,32p' "$0"
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
  shift
done

# Well-known snarkOS development keys (ChaChaRng seed used by `--dev`).
DEV_KEY_0_PRIVATE="APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH"
DEV_KEY_0_ADDRESS="aleo1rhgdu77hgyqd3xjj8ucu3jj9r2krwz6mnzyd80gncr5fxcwlh5rsvzp9px"
DEV_KEY_1_PRIVATE="APrivateKey1zkp2RWGDcde3efb89rjhME1VYA8QMxcxep5DShNBR6n8Yjh"
DEV_KEY_1_ADDRESS="aleo1s3ws5tra87fjycnjrwsjcrnw2qxr8jfqqdugnf0xzqqw29q9m5pqem2u4t"

dev_key_private() {
  case "$1" in
    0) echo "$DEV_KEY_0_PRIVATE" ;;
    1) echo "$DEV_KEY_1_PRIVATE" ;;
    *) log "❌ Unsupported --dev-key $1 (supported: 0, 1)"; exit 1 ;;
  esac
}

dev_key_address() {
  case "$1" in
    0) echo "$DEV_KEY_0_ADDRESS" ;;
    1) echo "$DEV_KEY_1_ADDRESS" ;;
    *) log "❌ Unsupported --dev-key $1 (supported: 0, 1)"; exit 1 ;;
  esac
}

SNARKOS_ROOT="$PWD"
AMM_V3_ROOT="${AMM_V3_ROOT:-$SNARKOS_ROOT/../amm-v3}"
NETWORK_ID="${NETWORK_ID:-0}"
NETWORK="$(get_network_name "$NETWORK_ID")"
ENDPOINT="${ENDPOINT:-http://127.0.0.1:3030}"
PRIORITY_FEE="${PRIORITY_FEE:-1000000000}"  # 1000 credits — covers the large AMM deployment.
AMM_PROGRAM="${AMM_PROGRAM:-shield_swap.aleo}"
FREEZELIST_PROGRAM="${FREEZELIST_PROGRAM:-shield_swap_freezelist.aleo}"
FREEZELIST_BLOCK_WINDOW="${FREEZELIST_BLOCK_WINDOW:-100}"

# `PRIVATE_KEY` is the snarkOS development key used to sign deploys/executes.
# `DEPLOYER_ADDRESS` is always --dev-key 0, matching the hardcoded DEPLOYER
# constants in shield_swap_freezelist and the test-token ADMIN addresses.
PRIVATE_KEY="$(dev_key_private "$DEV_KEY")"
DEPLOYER_ADDRESS="$(dev_key_address 0)"
DEPLOYING_ADDRESS="$(dev_key_address "$DEV_KEY")"

TOKEN_WRAPPERS=(
  test_token_a
  test_token_b
  test_token_c
  test_token_d
)

# Peripheral programs deployed after the AMM and its static dependencies.
ROUTER_PACKAGES=(
  shield_swap_lp_router
  shield_swap_router
)

ALL_PROGRAMS=(
  test_token_a.aleo
  test_token_b.aleo
  test_token_c.aleo
  test_token_d.aleo
  shield_swap_multisig_core.aleo
  "$FREEZELIST_PROGRAM"
  "$AMM_PROGRAM"
  shield_swap_lp_router.aleo
  shield_swap_router.aleo
)

if [[ ! -d "$AMM_V3_ROOT" ]]; then
  log "❌ amm-v3 repo not found at $AMM_V3_ROOT (set AMM_V3_ROOT)"
  exit 1
fi
AMM_V3_ROOT="$(cd "$AMM_V3_ROOT" && pwd)"

require_cmd leo
require_cmd snarkos
require_cmd curl
require_cmd jq

run() {
  if (( DRY_RUN )); then
    printf '  $ %s\n' "$*"
  else
    "$@"
  fi
}

rest_url() {
  local route="$1"
  echo "$ENDPOINT/v2/$NETWORK/$route"
}

devnet_reachable() {
  curl -sf -m 5 "$(rest_url "block/height/latest")" >/dev/null
}

program_deployed() {
  local status
  status=$(curl -s -o /dev/null -w "%{http_code}" "$(rest_url "program/$1")" || true)
  [[ "$status" == "200" ]]
}

mapping_value() {
  local key="$1" mapping="$2" program="$3"
  curl -sf -m 5 "$(rest_url "program/$program/mapping/$mapping/$key")" 2>/dev/null || true
}

fetch_consensus_heights() {
  local version_json
  version_json=$(curl -sf "$(rest_url "version")")
  jq -r '.consensus_heights | join(",")' <<< "$version_json"
}

leo_devnet_flags() {
  local -a flags=(
    --network "$NETWORK"
    --endpoint "$ENDPOINT"
    --private-key "$PRIVATE_KEY"
    --priority-fees "$PRIORITY_FEE"
    --consensus-heights "$CONSENSUS_VERSION_HEIGHTS"
    --devnet
    --yes
    --broadcast
    --max-wait 60
    --blocks-to-check 30
  )
  if [[ -n "${CONSENSUS_VERSION:-}" ]]; then
    flags+=(--consensus-version "$CONSENSUS_VERSION")
  fi
  printf '%s\n' "${flags[@]}"
}

deploy_leo_package() {
  local package_dir="$1"
  local program_id="$2"
  shift 2
  local -a extra=("$@")

  if program_deployed "$program_id" && [[ "${FORCE_AMM:-0}" != "1" ]]; then
    log "✅ $program_id already deployed — skipping"
    return 0
  fi

  log "Building $program_id in $package_dir"
  ( cd "$package_dir" && run leo build )

  local -a skip_args=()
  local dep
  for dep in shield_swap_multisig_core.aleo "$FREEZELIST_PROGRAM" "$AMM_PROGRAM" u256 IARC20; do
    if program_deployed "$dep"; then
      skip_args+=(--skip "$dep")
    fi
  done

  log "Deploying $program_id"
  local -a flags=()
  while IFS= read -r line; do
    [[ -n "$line" ]] && flags+=("$line")
  done < <(leo_devnet_flags)

  if ! (
    cd "$package_dir" && \
    run leo deploy "${flags[@]}" "${skip_args[@]+"${skip_args[@]}"}" "${extra[@]+"${extra[@]}"}"
  ); then
    local _
    for _ in 1 2 3 4 5 6 7 8; do
      program_deployed "$program_id" && break
      sleep 3
    done
    program_deployed "$program_id" || { log "❌ Failed to deploy $program_id (not on-chain)"; exit 1; }
    log "✅ $program_id deployed (recovered from broadcast race)"
  else
    log "✅ $program_id deployed"
  fi
}

snarkos_execute() {
  local program_id="$1"
  local function_name="$2"
  shift 2

  if (( DRY_RUN )); then
    printf '  $ snarkos developer execute --dev-key %s --network %s %s %s %s\n' \
      "$DEV_KEY" "$NETWORK_ID" "$program_id" "$function_name" "$*"
    return 0
  fi

  snarkos developer execute \
    --dev-key "$DEV_KEY" \
    --network "$NETWORK_ID" \
    --endpoint "$ENDPOINT" \
    --broadcast \
    --wait \
    --timeout 180 \
    "$program_id" "$function_name" "$@"
}

log "Endpoint: $ENDPOINT  Network: $NETWORK (id=$NETWORK_ID)  --dev-key $DEV_KEY ($DEPLOYING_ADDRESS)"
log "amm-v3: $AMM_V3_ROOT"
devnet_reachable || { log "❌ Devnet unreachable at $ENDPOINT"; exit 1; }
log "✅ Devnet reachable (height $(curl -sf "$(rest_url "block/height/latest")"))"

CONSENSUS_VERSION_HEIGHTS="$(fetch_consensus_heights)"
if [[ ! "$CONSENSUS_VERSION_HEIGHTS" =~ ^[0-9]+(,[0-9]+)*$ ]]; then
  log "❌ Failed to read consensus heights from $ENDPOINT: $CONSENSUS_VERSION_HEIGHTS"
  exit 1
fi
CONSENSUS_VERSION="$(get_consensus_version 0 "$NETWORK")"
export CONSENSUS_VERSION_HEIGHTS
log "Using CONSENSUS_VERSION_HEIGHTS=$CONSENSUS_VERSION_HEIGHTS (version $CONSENSUS_VERSION)"

if [[ "$DEV_KEY" != "0" ]]; then
  log "⚠️ Deploying with --dev-key $DEV_KEY; freezelist constructor requires --dev-key 0 as program owner"
fi

# ----- 1. Standalone test tokens (no dependencies). -----
if (( AMM_ONLY == 0 )); then
  for w in "${TOKEN_WRAPPERS[@]}"; do
    deploy_leo_package "$AMM_V3_ROOT/token-wrappers/$w" "${w}.aleo"
  done
fi

# ----- 2. AMM and static dependencies (multisig core, freeze list). -----
deploy_leo_package "$AMM_V3_ROOT" "$AMM_PROGRAM"

# ----- 3. Peripheral routers (import the AMM). -----
if (( AMM_ONLY == 0 )); then
  for router in "${ROUTER_PACKAGES[@]}"; do
    deploy_leo_package "$AMM_V3_ROOT/$router" "${router}.aleo"
  done
fi

# ----- 4. Initialize freeze-list state. -----
current_root=$(mapping_value "1u8" "freeze_list_root" "$FREEZELIST_PROGRAM")
if [[ -z "$current_root" || "$current_root" == "null" ]]; then
  log "Initializing $FREEZELIST_PROGRAM"
  snarkos_execute "$FREEZELIST_PROGRAM" initialize "$DEPLOYER_ADDRESS" "${FREEZELIST_BLOCK_WINDOW}u32"
  log "✅ $FREEZELIST_PROGRAM initialized"
else
  log "✅ $FREEZELIST_PROGRAM already initialized"
fi

admin_role=$(mapping_value "$DEPLOYER_ADDRESS" "address_to_role" "$FREEZELIST_PROGRAM")
admin_role=${admin_role//\"/}
if [[ "$admin_role" != "24u16" ]]; then
  log "Granting freeze-list manager role to $DEPLOYER_ADDRESS"
  snarkos_execute "$FREEZELIST_PROGRAM" update_role "$DEPLOYER_ADDRESS" "24u16"
  log "✅ $DEPLOYER_ADDRESS granted manager + freezelist-manager roles"
else
  log "✅ $DEPLOYER_ADDRESS already has manager + freezelist-manager roles"
fi

# ----- 5. Confirm every program is on-chain. -----
missing=0
for program_id in "${ALL_PROGRAMS[@]}"; do
  if (( AMM_ONLY )) && [[ "$program_id" != "$AMM_PROGRAM" && "$program_id" != "$FREEZELIST_PROGRAM" && "$program_id" != "shield_swap_multisig_core.aleo" ]]; then
    continue
  fi
  if program_deployed "$program_id"; then
    log "✅ $program_id is on-chain"
  else
    log "❌ $program_id is missing on-chain"
    missing=1
  fi
done

if (( missing && DRY_RUN == 0 )); then
  log "❌ Not all amm-v3 programs were deployed"
  exit 1
fi

log "✅ All amm-v3 deployments complete."
