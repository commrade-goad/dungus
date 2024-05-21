pub enum Command {
    PAUSED,
    PLAY(String),
    STOP,
    VOLUP(f32),
    VOLDOWN(f32),
    VOL(f32),
    EXIT,
    LOOP(bool),
}

pub struct Media {
    pub title: String,
    pub artist: String,
}
