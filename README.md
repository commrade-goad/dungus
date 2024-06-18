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
- [x] multi file (still no folder support for now)
- [x] change to use `Command::PLAY` to play audio
- [x] working stop
- [x] bind `j` and `k` to change tracks
- [ ] the media list
- [ ] config file? (maybe not)
- [x] loop
- [x] move metadata extract to main threads

## NOTE
This project is still Work In Progress!
