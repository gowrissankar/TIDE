typeset -g TIDE_WAKE=0
TRAPUSR1() {
  TIDE_WAKE=1
  zle accept-line
}
precmd() {
  if (( TIDE_WAKE )); then
    TIDE_WAKE=0
    echo "TIDE LAUNCHED"
  fi
}
