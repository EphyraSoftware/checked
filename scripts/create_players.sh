#!/usr/bin/env bash

RED='\033[0;33m'
BLUE='\033[0;34m'
CLEAR='\033[0m'

set -euo pipefail

pushd checked_cli || exit
trap 'echo -e "${RED}Error running setup${CLEAR}" && popd' SIGINT EXIT

echo -e "${BLUE}Creating player a${CLEAR}"

rm -rf /tmp/checked-tests/player-a
mkdir -p /tmp/checked-tests/player-a
cargo run --quiet -- generate --name player-a --password abc1 --distribute true --config-dir /tmp/checked-tests/player-a

echo -e "${BLUE}Creating player b${CLEAR}"

rm -rf /tmp/checked-tests/player-b
mkdir -p /tmp/checked-tests/player-b
cargo run --quiet -- generate --name player-b --password abc2 --distribute true --config-dir /tmp/checked-tests/player-b

popd || exit
trap - EXIT
