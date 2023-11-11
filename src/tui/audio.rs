use lofty::{Probe, TaggedFileExt, Accessor};
use rodio::{Decoder, OutputStream};
use std::fs;
use std::io::BufReader;
use std::path;

pub struct Media {
    pub title: String,
    pub artist: String,
}

pub fn play_sound(path_to_file: String) -> Option<rodio::Sink> {
    match &path_to_file[..] {
        "none" => {
            return None;
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
            return Some(sink);
        }
    }
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
    }
}
