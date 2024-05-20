# `snakegame`

The classic game Snake, written in Rust. This program runs entirely in a terminal. It is designed for both psuedo-terminals (that is, X or Wayland terminals), and Linux TTYs (the final program will be run in a TTY). This program is being written for the 2024 Onslow College open evening.

### TODO
- Speeding up the snake over time (undecided).
- Continuing work on the leaderboard.
- Documenting the remainder of the code (everything except `src/snake.rs`).
- Leaderboard: when "You!" is being put on leaderboard to show the user's score, change the position to an actual number, don't keep it on 10
- Fixing terminal resize issues :(
- Second colour of fruit (that speeds)
- Some kind of audio feedback.
- Get rid of `O_NONBLOCK` on stdin