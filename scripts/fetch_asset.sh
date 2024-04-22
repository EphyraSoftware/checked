#!/usr/bin/env bash

RED='\033[0;33m'
BLUE='\033[0;34m'
CLEAR='\033[0m'

set -euo pipefail

pushd checked_cli || exit
trap 'echo -e "${RED}Error running fetch${CLEAR}" && popd' SIGINT EXIT

echo -e "${BLUE}Fetching file as player a${CLEAR}"

cargo run --quiet -- fetch https://raw.githubusercontent.com/EphyraSoftware/checked/main/README.md --name player-a --password abc1 --config-dir /tmp/checked-tests/player-a --output /tmp/checked-tests/player-a/README.md --allow-no-signatures true --sign true

echo -e "${BLUE}Fetching file as player b${CLEAR}"

echo -e "${BLUE}hello player b${CLEAR}" > /tmp/checked-tests/player-b/file.txt
cargo run --quiet -- fetch https://raw.githubusercontent.com/EphyraSoftware/checked/main/README.md --name player-b --password abc2 --config-dir /tmp/checked-tests/player-b --output /tmp/checked-tests/player-b/README.md --allow-no-signatures true --sign true

popd || exit
trap - SIGINT EXIT
