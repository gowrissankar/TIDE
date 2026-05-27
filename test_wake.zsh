typeset -g TIDE_WAKE=0
TRAPUSR1() {
  TIDE_WAKE=1
}
precmd() {
  if (( TIDE_WAKE )); then
    TIDE_WAKE=0
    echo "TIDE LAUNCHED"
  fi
}
echo "My PID: $$"
(sleep 2; kill -USR1 $$) &
sleep 5
