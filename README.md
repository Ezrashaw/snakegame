# `snakegame`

The classic game Snake, written in Rust. This program runs entirely in a
terminal. It is designed for both psuedo-terminals (that is, X or Wayland
terminals), and Linux TTYs (the final program will be run in a TTY). This
program is being written for the 2024 Onslow College open evening.
Additionally, this program depends only on Rust's `libstd` and `libc` (the
latter itself a dependency of the former).

### TODO
- Continuing work on the leaderboard.
- Documenting the remainder of the code (everything except `src/snake.rs`).
- Leaderboard: when "You!" is being put on leaderboard to show the user's
  score, change the position to an actual number, don't keep it on 10
- Fixing terminal resize issues :(
- Some kind of audio feedback.

## Instructions for running on bare metal linux

1. Tell the kernel not to write `prink` messages to the console:
   `echo 1 > /proc/sys/kernel/printk`
2. Disable the shift and capslock keys (both break WASD, and we don't need them anyway):
   `loadkeys key.bmap` (needs to be created from the file in `contrib/` on a host system)
3. Load the patched console font:
   `setfont contrib/default16x32-patched.psfu`
