# TIDE Bash Integration
# Source this file in your ~/.bashrc

# Check if we are in a top-level interactive shell
if [[ "$-" == *i* ]] && [[ "$SHLVL" -eq 1 ]]; then
    # Start the tide-watch daemon
    tide-watch --tty "$(tty)" &
    export TIDE_WATCH_PID=$!
    disown

    # Send PROMPT_VISIBLE (SIGUSR1) before each prompt
    tide_prompt_command() {
        if [[ -n "$TIDE_WATCH_PID" ]]; then
            kill -USR1 "$TIDE_WATCH_PID" 2>/dev/null
        fi
    }

    # Send COMMAND_STARTED (SIGUSR2) before executing a command
    # Note: DEBUG trap runs before every command, including internal shell commands.
    # This is an accepted V1 limitation.
    tide_debug_trap() {
        if [[ -n "$TIDE_WATCH_PID" ]]; then
            kill -USR2 "$TIDE_WATCH_PID" 2>/dev/null
        fi
    }

    # Hook into bash PROMPT_COMMAND
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="tide_prompt_command"
    else
        PROMPT_COMMAND="$PROMPT_COMMAND; tide_prompt_command"
    fi

    # Hook into bash DEBUG trap
    trap 'tide_debug_trap' DEBUG
fi
