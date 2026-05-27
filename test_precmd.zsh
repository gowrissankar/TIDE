typeset -g TIDE_WAKE=0
TRAPUSR1() {
  TIDE_WAKE=1
  echo "TRAP RUN"
}
precmd() {
  if (( TIDE_WAKE )); then
    TIDE_WAKE=0
    echo "PRECMD RUN: TIDE LAUNCHED"
  fi
}
