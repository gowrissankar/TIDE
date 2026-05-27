trap 'echo "USR1 hit"; sleep 2' USR1
echo "My PID: $$"
(sleep 2; kill -USR1 $$; exit 0) &
sleep 5
