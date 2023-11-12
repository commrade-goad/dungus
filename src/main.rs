mod tui;
use std::env;
use std::process;
use tui::start_window;

fn main() {
    let user_args: Vec<String> = env::args().collect();
    if user_args.len() < 2 {
        eprintln!("ERR: Not Enought args!");
        process::exit(1);
    }
    start_window(&user_args[1]);
}
