mod app;
mod input;
mod life;
mod render;
mod screen;

use std::env;
use app::App;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args.len() == 3 && args[1] == "--init" {
            let shell = &args[2];
            if shell == "zsh" {
                println!("{}", r#"# TIDE Zsh Integration
# Source this file in your ~/.zshrc

if [[ -o interactive ]] && [[ "$SHLVL" -eq 1 ]]; then
    tide-watch --tty "$(tty)" &
    export TIDE_WATCH_PID=$!

    tide_precmd() {
        if [[ -n "$TIDE_WATCH_PID" ]]; then
            kill -USR1 "$TIDE_WATCH_PID" 2>/dev/null
        fi
    }

    tide_preexec() {
        if [[ -n "$TIDE_WATCH_PID" ]]; then
            kill -USR2 "$TIDE_WATCH_PID" 2>/dev/null
        fi
    }

    autoload -Uz add-zsh-hook
    add-zsh-hook precmd tide_precmd
    add-zsh-hook preexec tide_preexec
fi"#);
                return Ok(());
            } else if shell == "bash" {
                println!("{}", r#"# TIDE Bash Integration
# Source this file in your ~/.bashrc

if [[ "$-" == *i* ]] && [[ "$SHLVL" -eq 1 ]]; then
    tide-watch --tty "$(tty)" &
    export TIDE_WATCH_PID=$!
    disown

    tide_prompt_command() {
        if [[ -n "$TIDE_WATCH_PID" ]]; then
            kill -USR1 "$TIDE_WATCH_PID" 2>/dev/null
        fi
    }

    tide_debug_trap() {
        if [[ -n "$TIDE_WATCH_PID" ]]; then
            kill -USR2 "$TIDE_WATCH_PID" 2>/dev/null
        fi
    }

    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="tide_prompt_command"
    else
        PROMPT_COMMAND="$PROMPT_COMMAND; tide_prompt_command"
    fi

    trap 'tide_debug_trap' DEBUG
fi"#);
                return Ok(());
            } else {
                eprintln!("Unsupported shell: {}", shell);
                std::process::exit(1);
            }
        } else {
            eprintln!("Usage: tide [--init <zsh|bash>]");
            std::process::exit(1);
        }
    }

    let mut app = App::new();

    // Register signal handlers for clean terminal restoration
    let term_now = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    for sig in &[signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT, signal_hook::consts::SIGQUIT, signal_hook::consts::SIGHUP] {
        let _ = signal_hook::flag::register(*sig, std::sync::Arc::clone(&term_now));
    }

    // Spawn a thread to wait for the signal so we can cleanup and exit immediately,
    // rather than waiting for the 100ms poll_event timeout.
    std::thread::spawn(move || {
        while !term_now.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        crate::screen::TerminalSession::cleanup();
        std::process::exit(0);
    });

    app.run()
}
