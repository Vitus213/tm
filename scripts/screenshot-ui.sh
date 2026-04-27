#!/usr/bin/env bash
set -euo pipefail

SCREENSHOT_DIR="${1:-/tmp/tm-screenshots}"
mkdir -p "$SCREENSHOT_DIR"

GRIM="/nix/store/qdwm2jfh23mb5v7kbrd54czy980xfq6a-grim-1.5.0/bin/grim"

echo "Building tm-ui..."
cargo build -p tm-ui --quiet

echo "Starting tm-daemon..."
cargo run -p tm-daemon --quiet &
DAEMON_PID=$!
sleep 3

PAGES=("Overview" "Charts" "Data" "Apps" "Websites" "Categories" "Settings")

for page in "${PAGES[@]}"; do
    echo "Capturing $page..."
    TM_UI_PAGE="$page" cargo run -p tm-ui --quiet &
    UI_PID=$!
    sleep 3
    $GRIM "$SCREENSHOT_DIR/$page.png" || echo "  WARNING: screenshot failed for $page"
    kill $UI_PID 2>/dev/null || true
    sleep 0.5
done

kill $DAEMON_PID 2>/dev/null || true
echo "Screenshots saved to $SCREENSHOT_DIR"
ls -la "$SCREENSHOT_DIR"
