# TIDE

**T**erminal **I**dle **D**isplay **E**ngine

Terminal screensaver. Activates when your shell prompt has been idle. Runs Conway's Game of Life in fullscreen. Any keypress exits and returns your terminal exactly as it was.

---

## Install

```sh
cargo install --path .
```

Both binaries install:
- `tide` — the screensaver runtime
- `tide-watch` — the idle watcher (sleeps, then signals shell)

---

## Shell Setup

TIDE works by hooking into your shell's prompt lifecycle.

`tide --init <shell>` prints the hook snippet. You eval it in your rc file.

### Zsh

```sh
echo 'eval "$(tide --init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

### Bash

```sh
echo 'eval "$(tide --init bash)"' >> ~/.bashrc
source ~/.bashrc
```

That's it. Open a new terminal or source your rc.

---

## How It Works

```
shell prompt visible
    │
    ▼
tide-watch --timeout 300 --pid $$    (sleeping in background)
    │
user runs a command?
  YES → shell kills tide-watch → command runs normally
  NO  → timeout hits
            │
            ▼
         kill -USR1 $$ (signal shell)
            │
         shell catches SIGUSR1 trap
            │
            ▼
         exec tide natively in foreground
         fullscreen Conway
            │
         any keypress → exit
            │
            ▼
         back to shell prompt
```

The shell hooks manage the lifecycle:
- `precmd` / `PROMPT_COMMAND` — kills old watcher, starts fresh one on every prompt
- `preexec` / `DEBUG trap` — kills watcher before any command runs (so tide never fires inside vim, ssh, top, etc.)

---

## Running Manually

Run the screensaver directly any time:

```sh
tide
```

Test the watcher with a short timeout (always use `&` — it's a background-only process):

```sh
tide-watch --timeout 10 --pid $$ &
```

This starts a 10s countdown in the background. Your prompt returns immediately.
After 10s of no commands, `tide-watch` sends a signal to your shell (`$$`), and `tide` launches. Run any command to reset the timer.

> **Note:** Running `tide-watch` without `&` blocks your shell until the timeout
> expires. That's expected — it's just sleeping. Use `&` when testing.

Check the hook output:

```sh
tide --init zsh
tide --init bash
```

---

## Timeout

Default: **300 seconds** (5 minutes).

To change, edit the hook in your rc file:

```zsh
tide-watch --timeout 120 --pid $$ &!   # 2 minutes
```

---

## Uninstall

Remove the `eval` line from your rc file, then:

```sh
cargo uninstall tide
```

---

## Limitations (V1)

- **Prompt idle only** — detects idle shell prompt, not full terminal inactivity (no readline tracking)
- **Wake key consumed** — the keypress that exits TIDE is not delivered to the shell. Normal screensaver behavior.
- **Nested shells** — unsupported in V1. Each shell instance needs its own hooks.
- **Slow typist edge case** — if timeout fires while you're mid-keystroke, TIDE launches. Press any key to exit.

---

## Requirements

- Rust toolchain (`cargo install --path .`)
- `tide` and `tide-watch` in `PATH` (handled by cargo install)
- Zsh or Bash
