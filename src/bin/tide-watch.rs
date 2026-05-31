use libc::{SIGCHLD, SIGTERM, SIGUSR1, SIGUSR2, c_int};
use signal_hook::iterator::Signals;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Disarmed,
    Armed,
    Rendering(libc::pid_t),
}

enum Event {
    Signal(c_int),
}

// get_tty removed

fn acquire_lock(tty: &str) -> std::io::Result<()> {
    let tty_name = tty.replace("/", "_");
    let lock_path = format!("/tmp/tide-{}.pid", tty_name);

    // Verify existing lock
    if let Ok(mut file) = fs::File::open(&lock_path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            if let Ok(pid) = content.trim().parse::<i32>() {
                // Check if process exists
                let kill_result = unsafe { libc::kill(pid, 0) };
                if kill_result == 0
                    || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
                {
                    std::process::exit(0);
                } else {
                    let _ = fs::remove_file(&lock_path);
                }
            }
        }
    }

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&lock_path)
    {
        Ok(mut file) => {
            let pid = std::process::id();
            writeln!(file, "{}", pid)?;
            Ok(())
        }
        Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {
            std::process::exit(0);
        }
        Err(e) => Err(e),
    }
}

fn reap_children() {
    loop {
        let mut status = 0;
        let pid = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
        if pid <= 0 {
            break;
        }
    }
}

use std::os::unix::io::AsRawFd;

fn main() -> std::io::Result<()> {
    unsafe {
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
    }

    if let Ok(dev_null) = OpenOptions::new().read(true).write(true).open("/dev/null") {
        unsafe {
            libc::dup2(dev_null.as_raw_fd(), 0);
            libc::dup2(dev_null.as_raw_fd(), 1);
            libc::dup2(dev_null.as_raw_fd(), 2);
        }
    }

    let mut args = env::args();
    let mut tty_opt = None;
    let mut timeout_opt = None;
    while let Some(arg) = args.next() {
        if arg == "--tty" {
            tty_opt = args.next();
        } else if arg == "--timeout" {
            timeout_opt = args.next();
        }
    }

    let tty = match tty_opt {
        Some(t) => t,
        None => {
            std::process::exit(1);
        }
    };

    acquire_lock(&tty)?;

    //set time here

    let timeout_secs: u64 = timeout_opt.and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        env::var("TIDE_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5)
    });
    let timeout = Duration::from_secs(timeout_secs);

    let (tx, rx) = mpsc::channel();
    let mut signals = Signals::new(&[SIGUSR1, SIGUSR2, SIGCHLD, SIGTERM])?;

    thread::spawn(move || {
        for sig in signals.forever() {
            let _ = tx.send(Event::Signal(sig));
        }
    });

    let mut state = State::Disarmed;

    loop {
        match state {
            State::Disarmed => {
                if let Ok(Event::Signal(sig)) = rx.recv() {
                    match sig {
                        SIGUSR1 => {
                            state = State::Armed;
                        }
                        SIGCHLD => {
                            reap_children();
                        }
                        SIGTERM => {
                            break;
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            State::Armed => {
                match rx.recv_timeout(timeout) {
                    Ok(Event::Signal(sig)) => {
                        match sig {
                            SIGUSR1 => {
                                // Re-arm, let loop restart timeout
                            }
                            SIGUSR2 => {
                                state = State::Disarmed;
                            }
                            SIGCHLD => {
                                reap_children();
                            }
                            SIGTERM => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
                        let tide_path = exe_dir.join("tide");

                        if let Ok(tty_file) = OpenOptions::new().read(true).write(true).open(&tty) {
                            match Command::new(tide_path)
                                .stdin(std::process::Stdio::from(tty_file.try_clone().unwrap()))
                                .stdout(std::process::Stdio::from(tty_file.try_clone().unwrap()))
                                .stderr(std::process::Stdio::from(tty_file.try_clone().unwrap()))
                                .spawn() 
                            {
                                Ok(child) => {
                                    let fd = tty_file.as_raw_fd();
                                    let shell_pgrp = unsafe { libc::tcgetpgrp(fd) };
                                    unsafe { libc::tcsetpgrp(fd, child.id() as libc::pid_t); }
                                    state = State::Rendering(shell_pgrp);
                                }
                                Err(_e) => {
                                    state = State::Disarmed;
                                }
                            }
                        } else {
                            // If we can't open the TTY, we can't render. Backoff.
                            state = State::Disarmed;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
            State::Rendering(shell_pgrp) => {
                if let Ok(Event::Signal(sig)) = rx.recv() {
                    match sig {
                        SIGCHLD => {
                            if let Ok(tty_file) = OpenOptions::new().read(true).write(true).open(&tty) {
                                unsafe { libc::tcsetpgrp(tty_file.as_raw_fd(), shell_pgrp); }
                            }
                            reap_children();
                            state = State::Disarmed;
                        }
                        SIGTERM => {
                            if let Ok(tty_file) = OpenOptions::new().read(true).write(true).open(&tty) {
                                unsafe { libc::tcsetpgrp(tty_file.as_raw_fd(), shell_pgrp); }
                            }
                            reap_children(); 
                            break;
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}
