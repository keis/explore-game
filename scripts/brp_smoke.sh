#!/usr/bin/env bash
# Smoke-test the Bevy Remote Protocol endpoint of a running game.
# Usage: BRP_PORT=15702 ./scripts/brp_smoke.sh
# The game must be running with the `brp` feature:
#   cargo run --features brp
set -euo pipefail

PORT="${BRP_PORT:-15702}"
URL="http://localhost:${PORT}/"

brp() {
  local method="$1" params="$2"
  curl -sf -X POST "${URL}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"${method}\",\"params\":${params}}"
}

echo "== query Party entities =="
PARTIES=$(brp "world.query" '{"data":{"components":["explore_game::actor::component::Party"]}}' \
  | jq '.result | length')
echo "parties: ${PARTIES}"

echo "== screenshot =="
brp "brp_extras/screenshot" '{"path":"/tmp/brp_smoke.png"}' | jq .
ls -la /tmp/brp_smoke.png
