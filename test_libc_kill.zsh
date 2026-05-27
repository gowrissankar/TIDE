echo "My PID: $$"
TRAPUSR1() {
    echo "Got SIGUSR1, running in foreground: $(tty)"
}
cargo run --bin tide-watch -- --timeout 2 --pid $$ &
sleep 4
