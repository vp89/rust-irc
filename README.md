# Rust IRC

This is a toy project to practice writing Rust code.

View thread usage:

`ps -eL -q $(pidof rust-irc) | tail -n +2 | wc -l`

View socket/fd usage:

`lsof -Pan -p $(pidof rust-irc) -i`
