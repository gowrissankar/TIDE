//entry point for tide binary
//thin as possible , all logic lives in app.rs

//module declarations
mod app;
mod input;
mod life;
mod render;
mod screen;

use app::App;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    //check for --init <shell> flag
    //prints shell hook snippet to stdout , does NOT edit any files
    //user pipes it into eval or pastes it themselves → unix-native
    if args.len() == 3 && args[1] == "--init" {
        match args[2].as_str() {
            "zsh" => {
                print!("{}", ZSH_HOOK);
                return Ok(());
            }
            "bash" => {
                print!("{}", BASH_HOOK);
                return Ok(());
            }
            other => {
                eprintln!("tide: unknown shell '{}', supported: zsh, bash", other);
                std::process::exit(1);
            }
        }
    }

    //no flags = run the screensaver directly
    //this is how tide-watch calls us
    let mut app = App::new();
    app.run()
}

// =============================================================================
// shell hook snippets — emitted by tide --init <shell>
// just prints , never edits files
// user adds: eval "$(tide --init zsh)" to their rc
// =============================================================================

const ZSH_HOOK: &str = r#"
_TIDE_WATCH_PID=""
typeset -g TIDE_WAKE=0

TRAPUSR1() {
    # simply set state in the signal handler to avoid job-control suspension
    # if we do terminal output directly in the trap, zsh can suspend the watcher
    TIDE_WAKE=1
}

_tide_preexec() {
    # command starting — kill watcher so it won't fire during vim/ssh/etc
    if [[ -n "$_TIDE_WATCH_PID" ]]; then
        kill "$_TIDE_WATCH_PID" 2>/dev/null
        _TIDE_WATCH_PID=""
    fi
}

_tide_precmd() {
    # if watcher signaled us, run tide natively in the foreground lifecycle
    if (( TIDE_WAKE )); then
        TIDE_WAKE=0
        tide
    fi

    # prompt showing — kill stale watcher first, then start fresh
    if [[ -n "$_TIDE_WATCH_PID" ]]; then
        kill "$_TIDE_WATCH_PID" 2>/dev/null
    fi
    # &! = background + disown atomically (zsh-specific)
    # disown removes it from the job table entirely
    # shell never sees it start, never sees it end — no [1] spam, no terminated noise
    tide-watch --timeout 300 --pid $$ &!
    _TIDE_WATCH_PID=$!
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec _tide_preexec
add-zsh-hook precmd  _tide_precmd
"#;

const BASH_HOOK: &str = r#"
_TIDE_WATCH_PID=""
TIDE_WAKE=0

trap 'TIDE_WAKE=1' USR1

_tide_preexec() {
    # command starting — kill watcher
    if [[ -n "$_TIDE_WATCH_PID" ]]; then
        kill "$_TIDE_WATCH_PID" 2>/dev/null
        _TIDE_WATCH_PID=""
    fi
}

_tide_precmd() {
    if (( TIDE_WAKE == 1 )); then
        TIDE_WAKE=0
        tide
    fi

    # prompt showing — kill stale watcher, start fresh
    if [[ -n "$_TIDE_WATCH_PID" ]]; then
        kill "$_TIDE_WATCH_PID" 2>/dev/null
    fi
    # disown removes it from the job table immediately
    # shell won't track it, won't print [1] PID on start or terminated on death
    tide-watch --timeout 300 --pid $$ &
    _TIDE_WATCH_PID=$!
    disown $_TIDE_WATCH_PID
}

# safe append — does NOT overwrite existing PROMPT_COMMAND
PROMPT_COMMAND="${PROMPT_COMMAND:+${PROMPT_COMMAND}; }_tide_precmd"

# DEBUG trap fires before each command
# note: fires on internal commands too — bash limitation, v1 accepted
trap '_tide_preexec' DEBUG
"#;
