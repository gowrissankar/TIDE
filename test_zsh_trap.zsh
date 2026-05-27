TRAPUSR1() {
    echo "Running in foreground: $(tty)"
    sleep 2
    echo "Done"
}
echo "My PID: $$"
(sleep 2; kill -USR1 $$) &
sleep 5
