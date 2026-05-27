# TIDE Zsh Integration
# Source this file in your ~/.zshrc

# Check if we are in a top-level interactive shell
if [[ -o interactive ]] && [[ "$SHLVL" -eq 1 ]]; then
    # Start the tide-watch daemon
    #tide-watch --tty "$(tty)" &!
    tide-watch --tty "$(tty)" >/tmp/tide-watch.log 2>&1 &
    export TIDE_WATCH_PID=$!

    # Send PROMPT_VISIBLE (SIGUSR1) before each prompt
    tide_precmd() {
        if [[ -n "$TIDE_WATCH_PID" ]]; then
            kill -USR1 "$TIDE_WATCH_PID" 2>/dev/null
        fi
    }

    # Send COMMAND_STARTED (SIGUSR2) before executing a command
    tide_preexec() {
        if [[ -n "$TIDE_WATCH_PID" ]]; then
            kill -USR2 "$TIDE_WATCH_PID" 2>/dev/null
        fi
    }

    # Hook into zsh's precmd and preexec arrays safely
    autoload -Uz add-zsh-hook
    add-zsh-hook precmd tide_precmd
    add-zsh-hook preexec tide_preexec
fi
