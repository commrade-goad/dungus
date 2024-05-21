use crate::command::*;
use rodio::{Decoder, OutputStream};
use std::fs;
use std::io::BufReader;
use std::path;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time::Duration;

pub fn audio_server(
    receiver_clone: &Receiver<Command>,
    sender2_clone: &Sender<Command>,
) -> i8 {
    let mut is_loop: bool = false;
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).expect("ERR: Failed to create sink");
    loop {
        sleep(Duration::from_millis(100));
        match sender2_clone.send(Command::VOL(get_current_volume(&sink))) {
            Ok(_) =>(),
            Err(_) => ()
        }
        match sender2_clone.send(Command::LOOP(is_loop)) {
            Ok(_) =>(),
            Err(_) => ()
        }
        if sink.empty() {
            match sender2_clone.send(Command::STOP) {
                Ok(_) =>(),
                Err(_) => ()
            }
        }
        match receiver_clone.try_recv() {
            Ok(Command::PAUSED) => toggle_pause(&sink),
            Ok(Command::PLAY(val)) => {
                match &val[..] {
                    "none" => {}
                    _ => {
                        match path::Path::new(&val).is_file() {
                            true => {
                                let file = BufReader::new(fs::File::open(val).unwrap());
                                let source = Decoder::new(file).unwrap();
                                sink.append(source);
                            } 
                            false => {}
                        }
                    }
                };
            }
            Ok(Command::STOP) => {
                sink.clear();
            }
            Ok(Command::VOLUP(val)) => {
                if get_current_volume(&sink) <= 0.95 {
                    let value = get_current_volume(&sink) + val;
                    sink.set_volume(value);
                } else {
                    sink.set_volume(1.0);
                }
            }
            Ok(Command::VOLDOWN(val)) => {
                if get_current_volume(&sink) >= 0.05 {
                    let value = get_current_volume(&sink) - val;
                    sink.set_volume(value);
                } else {
                    sink.set_volume(0.0);
                }
            }
            Ok(Command::EXIT) => {
                return 0;
            }
            Ok(Command::LOOP(v)) => {
                is_loop = v;
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }
}

fn toggle_pause(sink: &rodio::Sink) {
    if sink.is_paused() {
        sink.play();
    } else {
        sink.pause();
    }
}

fn get_current_volume(sink: &rodio::Sink) -> f32 {
    return sink.volume();
}
