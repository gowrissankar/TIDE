source tide.zsh
sleep 1
ps -o pid,ppid,state,tty,cmd -C tide-watch
echo "Simulating prompt..."
kill -USR1 $TIDE_WATCH_PID
sleep 2
echo "After 2 seconds, state is:"
ps -o pid,ppid,state,tty,cmd -C tide-watch
