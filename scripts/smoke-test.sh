#!/bin/bash
# Smoke test: verify build artifacts exist and are valid

PASS=0
FAIL=0

check() {
    if eval "$2"; then
        echo "PASS: $1"
        PASS=$((PASS + 1))
    else
        echo "FAIL: $1"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== Prompter Smoke Test ==="
echo ""

# Check .dmg exists
DMG_PATH=$(ls src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1)
check ".dmg artifact exists" '[ -n "$DMG_PATH" ]'
if [ -n "$DMG_PATH" ]; then
    echo "      Found: $DMG_PATH"
fi

# Check binary exists
APP_BINARY="src-tauri/target/release/prompter"
check "Release binary exists" '[ -f "$APP_BINARY" ]'

# Check binary is valid Mach-O
if [ -f "$APP_BINARY" ]; then
    check "Binary is valid Mach-O executable" 'file "$APP_BINARY" | grep -q "Mach-O"'
fi

# Check .app bundle exists
APP_BUNDLE=$(ls -d src-tauri/target/release/bundle/macos/*.app 2>/dev/null | head -1)
check ".app bundle exists" '[ -n "$APP_BUNDLE" ]'

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
