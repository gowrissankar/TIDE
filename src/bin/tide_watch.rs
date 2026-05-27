//tide-watch : prompt-idle watcher
//dumb helper. sleep → signal shell. nothing else.

//owns:
// sleeping for --timeout seconds
// signaling shell when sleep finishes

//does NOT own:
// terminal state (no crossterm, no raw mode, no alt-screen)
// shell lifecycle
// IPC or complex state

//how it works:
// shell precmd (prompt visible) → kill old tide-watch → start fresh tide-watch
// shell preexec (cmd starting)  → kill tide-watch (stops it firing inside vim/ssh/etc)
// if timeout completes (prompt idle) → signal shell (SIGUSR1) to run tide in foreground

//limitations (v1 accepted):
// prompt idle only — not full terminal inactivity detection
// slow typist edge case : timeout could fire mid-keystroke , tide exits on wakeup key
// nested shells : unsupported / best-effort , hooks needed in each shell separately
// tiny race : user comes back exactly as timeout fires → brief tide flash , acceptable

//IMPORTANT : this process must be completely silent (no writes to stderr/stdout)
//background processes that write to the tty get SIGTTOU → suspended
//all output suppressed intentionally — errors are silently ignored

use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    //tide-watch [--timeout SECONDS] [--pid SHELL_PID]
    //default timeout : 300s (5 min)
    let args: Vec<String> = env::args().collect();
    let (timeout_secs, shell_pid) = parse_args(&args);

    //sleep for the configured idle timeout
    //if shell kills us before this completes (preexec) we just exit — correct behavior
    thread::sleep(Duration::from_secs(timeout_secs));

    //sleep done, prompt was idle
    if let Some(pid_str) = shell_pid {
        if let Ok(pid) = pid_str.parse::<i32>() {
            //v1 architecture fix: watcher doesn't spawn tide, it notifies shell.
            //use native libc::kill instead of spawning /bin/kill to avoid any
            //subprocess stdio/process-group inheritance that triggers SIGTTOU
            unsafe {
                let _rc = libc::kill(pid, libc::SIGUSR1);
                // silently ignore error, background process
            }
        }
    }
    // Note: If no --pid is provided, we do absolutely nothing.
    // Spawning tide directly from a background process causes SIGTTOU.
}

fn parse_args(args: &[String]) -> (u64, Option<String>) {
    let mut timeout = 300;
    let mut pid = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--timeout" {
            if let Some(val) = args.get(i + 1) {
                if let Ok(n) = val.parse::<u64>() {
                    timeout = n;
                }
            }
            i += 2;
        } else if args[i] == "--pid" {
            if let Some(val) = args.get(i + 1) {
                pid = Some(val.clone());
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    
    (timeout, pid)
}
