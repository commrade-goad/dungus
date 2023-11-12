# DUNGUS
a simple curses music player written in rust.

## BUILD
```shell
cargo build --release
```

## USAGE
```shell
dungus music.mp3
```
- for now it only support 1 file.
- `p` to toggle pause and play
- `q` to quit
- `[` volume down 5
- `]` volume up 5

## TODO
- [ ] multi file
- [ ] change to use `Command::PLAY` to play audio
- [ ] working stop
- [ ] bind `j` and `k` to change tracks
- [ ] config file? (maybe not)
- [ ] loop
- [ ] move metadata extract to main threads

## NOTE
This project is still Work In Progress!
