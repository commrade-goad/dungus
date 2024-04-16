use lofty::{Accessor, Probe, TaggedFileExt};
use rodio::{Decoder, OutputStream};
use std::fs;
use std::io::BufReader;
use std::path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct Media {
    pub title: String,
    pub artist: String,
}

pub enum Command {
    PAUSED,
    PLAY(String),
    STOP,
    VOLUP(f32),
    VOLDOWN(f32),
    VOL(f32),
}

pub fn play_sound(
    path_to_file: String,
    receiver_clone: Arc<Mutex<Receiver<Command>>>,
    sender2_clone: Arc<Mutex<Sender<Command>>>,
) -> Option<i32> {
    match &path_to_file[..] {
        "none" => {
            return Some(0);
        }
        _ => {}
    }
    match path::Path::new(&path_to_file).is_file() {
        false => {
            println!("Error : Cant read the specified file directory!");
            return None;
        }
        true => {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let file = BufReader::new(fs::File::open(path_to_file).unwrap());
            let source = Decoder::new(file).unwrap();
            let sink = rodio::Sink::try_new(&stream_handle).expect("ERR: Failed to create sink");
            sink.append(source);
            loop {
                sender2_clone.lock().unwrap().send(Command::VOL(get_current_volume(&sink))).unwrap();

                match receiver_clone.lock().unwrap().try_recv() {
                    Ok(Command::PAUSED) => toggle_pause(&sink),
                    Ok(Command::PLAY(val)) => todo!("PLAY"),
                    Ok(Command::STOP) => {
                        sink.clear();
                        return Some(0);
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
                    Ok(_) => {}
                    Err(_) => {}
                }
                if sink.empty() {
                    return Some(0);
                }
            }
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

pub fn read_metadata(path: &str) -> Media {
    let tagged_file = Probe::open(path)
        .expect("ERR: Failed to open the file.")
        .read()
        .expect("ERR: Failed to read the file.");

    let tag = match tagged_file.primary_tag() {
        Some(primary_tag) => primary_tag,
        None => tagged_file.first_tag().expect("ERROR: No tags found!"),
    };
    return Media {
        title: tag.title().as_deref().unwrap_or("None").to_string(),
        artist: tag.artist().as_deref().unwrap_or("None").to_string(),
    };
}
