# TIDE V1 — Persistent Watcher State-Machine Architecture

## Core Philosophy
TIDE is a terminal-native screensaver that activates when the shell prompt has been idle for a configurable amount of time.

It completely avoids complex terminal interception, tmux-like server/client architectures, or pseudo-TTY routing. Instead, it relies on a **Persistent Watcher Daemon** combined with **Foreground Process Group Handoff**.

## Components

### 1. The Shell Integration (`precmd` / `preexec`)
The shell runs zero polling logic. It simply emits two native signals using native hooks:
- **`precmd` (Prompt rendered)**: Sends `SIGUSR1` to the watcher. Signals that the user is idle at the prompt.
- **`preexec` (Command started)**: Sends `SIGUSR2` to the watcher. Signals that the user is running a command and the screensaver must not trigger.

### 2. `tide-watch` (The Persistent Daemon)
A lightweight background daemon spawned once per terminal session.
- Validates terminal ownership by taking the explicit `--tty` argument and creating a singleton lockfile (e.g., `/tmp/tide-_dev_pts_X.pid`).
- Connects to the shell lifecycle via `PDEATHSIG` so it cleanly dies when the terminal closes.
- Avoids terminal freezing/suspension by explicitly ignoring `SIGTTOU` and `SIGTTIN` and mapping standard IO to `/dev/null`.
- Manages an internal State Machine (`Disarmed`, `Armed`, `Rendering(shell_pgrp)`) using zero-polling `mpsc::channel` events.

### 3. `tide` (The Renderer)
A pure `crossterm` visualization binary.
- Never runs polling logic; it is spawned directly into execution by `tide-watch`.
- Enforces bulletproof terminal restoration via a `TerminalSession` RAII drop-guard, custom `signal-hook` interceptions for `SIGTERM`/`SIGINT`, and a `std::panic::set_hook`.

## The Foreground Handoff Magic (tcsetpgrp)
Because background processes cannot read terminal keystrokes, `tide` must become the foreground process to allow users to dismiss the screensaver via keyboard.
1. When `tide-watch` triggers, it grabs the shell's process group using `tcgetpgrp(tty_fd)`.
2. It spawns `tide` and immediately executes `tcsetpgrp(tty_fd, tide_pid)`.
3. `tide` now owns the terminal, blocking safely in `crossterm::poll()`, rendering to the alternate screen, and listening for keystrokes.
4. When `tide` exits (via keystroke or error), `tide-watch` catches `SIGCHLD`.
5. `tide-watch` then executes `tcsetpgrp(tty_fd, shell_pgrp)`, safely returning foreground ownership back to the interactive shell.

This allows TIDE to behave exactly like a native foreground command without breaking shell job control.
