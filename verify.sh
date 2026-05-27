#!/bin/bash

set -e

export PATH="$HOME/.cargo/bin:$PATH"

# =========================================================
# CONFIG
# =========================================================

TIDE_TIMEOUT=3

# =========================================================
# TERMINAL RECOVERY
# =========================================================

cleanup_terminal() {
    tput cnorm 2>/dev/null || true
    stty sane 2>/dev/null || true
}

trap cleanup_terminal EXIT

# =========================================================
# HARD CLEANUP
# =========================================================

echo ""
echo "========================================================="
echo "0. HARD CLEANUP"
echo "========================================================="

pkill -9 -f tide-watch 2>/dev/null || true
pkill -9 -x tide 2>/dev/null || true

rm -f /tmp/tide-*.pid 2>/dev/null || true
truncate -s 0 /tmp/tide-watch.log 2>/dev/null || true

sleep 1

# =========================================================
# START WATCHER
# =========================================================

echo ""
echo "========================================================="
echo "1. START WATCHER"
echo "========================================================="

tide-watch --tty "$(tty)" --timeout "$TIDE_TIMEOUT" &
TIDE_WATCH_PID=$!

sleep 1

# =========================================================
# VERIFY WATCHER HEALTH
# =========================================================

echo ""
echo "========================================================="
echo "2. VERIFY WATCHER HEALTH"
echo "========================================================="

ps -o pid,ppid,state,tty,cmd -p "$TIDE_WATCH_PID"

WATCHER_STATE=$(ps -o state= -p "$TIDE_WATCH_PID" | tr -d ' ')

if [[ "$WATCHER_STATE" != "S" ]]; then
    echo ""
    echo "ERROR: watcher is NOT sleeping healthy"
    echo "Expected: S"
    echo "Got: $WATCHER_STATE"
    exit 1
fi

echo ""
echo "SUCCESS: watcher healthy"

# =========================================================
# VERIFY LOCKFILE
# =========================================================

echo ""
echo "========================================================="
echo "3. VERIFY LOCKFILE"
echo "========================================================="

ls -l /tmp/tide-*

LOCK_PID=$(cat /tmp/tide-* 2>/dev/null || true)

echo ""
echo "Lockfile PID: $LOCK_PID"
echo "Watcher PID : $TIDE_WATCH_PID"

if [[ "$LOCK_PID" != "$TIDE_WATCH_PID" ]]; then
    echo ""
    echo "ERROR: lockfile PID mismatch"
    exit 1
fi

echo ""
echo "SUCCESS: lockfile valid"

# =========================================================
# ARM WATCHER
# =========================================================

echo ""
echo "========================================================="
echo "4. ARM WATCHER"
echo "========================================================="

kill -USR1 "$TIDE_WATCH_PID"

echo ""
echo "Waiting ${TIDE_TIMEOUT}s for renderer..."

sleep $((TIDE_TIMEOUT + 1))

# =========================================================
# VERIFY RENDERER LAUNCH
# =========================================================

echo ""
echo "========================================================="
echo "5. VERIFY RENDERER LAUNCH"
echo "========================================================="

TIDE_PID=$(ps -o pid,ppid,state,tty,cmd -C tide | \
    awk -v wpid="$TIDE_WATCH_PID" '$2 == wpid {print $1}')

if [[ -z "$TIDE_PID" ]]; then
    echo ""
    echo "ERROR: renderer did NOT launch"
    exit 1
fi

ps -o pid,ppid,state,tty,cmd -p "$TIDE_PID"

echo ""
echo "SUCCESS: renderer launched as PID $TIDE_PID"

# =========================================================
# VERIFY CLEAN EXIT
# =========================================================

echo ""
echo "========================================================="
echo "6. VERIFY CLEAN RENDERER EXIT"
echo "========================================================="

echo "Sending SIGTERM to renderer..."

kill -TERM "$TIDE_PID"

sleep 2

if ps -p "$TIDE_PID" >/dev/null 2>&1; then
    echo ""
    echo "ERROR: renderer failed to exit"
    exit 1
fi

echo ""
echo "SUCCESS: renderer exited cleanly"

# =========================================================
# VERIFY WATCHER SURVIVES
# =========================================================

echo ""
echo "========================================================="
echo "7. VERIFY WATCHER SURVIVES"
echo "========================================================="

ps -o pid,ppid,state,tty,cmd -p "$TIDE_WATCH_PID"

WATCHER_STATE=$(ps -o state= -p "$TIDE_WATCH_PID" | tr -d ' ')

if [[ "$WATCHER_STATE" != "S" ]]; then
    echo ""
    echo "ERROR: watcher unhealthy after renderer exit"
    echo "State: $WATCHER_STATE"
    exit 1
fi

echo ""
echo "SUCCESS: watcher survived renderer lifecycle"

# =========================================================
# VERIFY RETRIGGER
# =========================================================

echo ""
echo "========================================================="
echo "8. VERIFY RETRIGGER CYCLE"
echo "========================================================="

kill -USR1 "$TIDE_WATCH_PID"

echo ""
echo "Waiting ${TIDE_TIMEOUT}s for SECOND renderer..."

sleep $((TIDE_TIMEOUT + 1))

TIDE_PID2=$(ps -o pid,ppid,state,tty,cmd -C tide | \
    awk -v wpid="$TIDE_WATCH_PID" '$2 == wpid {print $1}')

if [[ -z "$TIDE_PID2" ]]; then
    echo ""
    echo "ERROR: renderer failed to relaunch"
    exit 1
fi

ps -o pid,ppid,state,tty,cmd -p "$TIDE_PID2"

echo ""
echo "SUCCESS: renderer relaunched as PID $TIDE_PID2"

kill -TERM "$TIDE_PID2"

sleep 2

# =========================================================
# VERIFY NO ZOMBIES
# =========================================================

echo ""
echo "========================================================="
echo "9. VERIFY NO ZOMBIES"
echo "========================================================="

ZOMBIES=$(ps -el | grep Z | grep tide || true)

if [[ -n "$ZOMBIES" ]]; then
    echo ""
    echo "ERROR: zombies detected"
    echo "$ZOMBIES"
    exit 1
fi

echo ""
echo "SUCCESS: no zombies"

# =========================================================
# VERIFY SINGLETON ENFORCEMENT
# =========================================================

echo ""
echo "========================================================="
echo "10. VERIFY SINGLETON ENFORCEMENT"
echo "========================================================="

tide-watch --tty "$(tty)" --timeout "$TIDE_TIMEOUT" &
sleep 1

WATCHER_COUNT=$(ps -C tide-watch --no-headers | wc -l)

ps -o pid,ppid,state,tty,cmd -C tide-watch

if [[ "$WATCHER_COUNT" -ne 1 ]]; then
    echo ""
    echo "ERROR: singleton enforcement failed"
    exit 1
fi

echo ""
echo "SUCCESS: singleton enforcement working"

# =========================================================
# FINAL CLEANUP
# =========================================================

echo ""
echo "========================================================="
echo "11. FINAL CLEANUP"
echo "========================================================="

pkill -TERM -P "$TIDE_WATCH_PID" 2>/dev/null || true
kill "$TIDE_WATCH_PID" 2>/dev/null || true

rm -f /tmp/tide-*.pid 2>/dev/null || true

cleanup_terminal

echo ""
echo "========================================================="
echo "ALL VERIFICATIONS PASSED"
echo "========================================================="