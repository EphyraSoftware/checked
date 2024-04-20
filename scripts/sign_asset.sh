#!/usr/bin/env bash

RED='\033[0;33m'
BLUE='\033[0;34m'
CLEAR='\033[0m'

set -euo pipefail

pushd checked_cli || exit
trap 'echo -e "${RED}Error running sign steps${CLEAR}" && popd' SIGINT EXIT

echo -e "${BLUE}Signing file as player a${CLEAR}"

echo -e "${BLUE}hello player a${CLEAR}" > /tmp/checked-tests/player-a/file.txt
cargo run --quiet -- sign https://example.com/file-a.txt --name player-a --password abc1 --config-dir /tmp/checked-tests/player-a --file /tmp/checked-tests/player-a/file.txt --distribute true

echo -e "${BLUE}Fetching file as player b${CLEAR}"

echo -e "${BLUE}hello player b${CLEAR}" > /tmp/checked-tests/player-b/file.txt
cargo run --quiet -- sign https://example.com/file-b.txt --name player-b --password abc2 --config-dir /tmp/checked-tests/player-b --file /tmp/checked-tests/player-b/file.txt --distribute true

popd || exit
trap - SIGINT EXIT
