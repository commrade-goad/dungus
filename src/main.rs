mod audio;
mod command;
mod tui;
use audio::audio_server;
use command::*;
use tui::start_window;
use std::env;
use std::process;
use std::sync::mpsc;
use std::thread;

fn main() {
    let user_args: Vec<String> = env::args().collect();
    if user_args.len() < 2 {
        eprintln!("ERR: Not Enought args!");
        process::exit(1);
    }
    // let file_path: String = user_args[1].clone();
    let file_path: Vec<String> = user_args[1..user_args.len()].to_vec();

    // Create a channel for sending and receiving messages
    let (thread_2_sender, thread_1_receiver) = mpsc::channel::<Command>();
    let (thread_1_sender, thread_2_receiver) = mpsc::channel::<Command>();

    // Spawn the first thread
    let thread1 = thread::spawn(move || {
        if start_window(file_path, &thread_1_receiver, &thread_1_sender) == 0 {
            println!("TUI thread exit successfully");
        }
    });

    if audio_server(&thread_2_receiver, &thread_2_sender) == 0{
        println!("Audio thread exit successfully");
    }

    // Wait for both threads to finish
    thread1.join().unwrap();
}
