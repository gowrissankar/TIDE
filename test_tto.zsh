trap 'echo "USR1 hit"' USR1
sleep 10 &
SLEEP_PID=$!
kill -USR1 $$
