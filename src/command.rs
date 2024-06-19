pub enum Command {
    PAUSED,
    PLAY(String),
    STOP,
    VOLUP(f32),
    VOLDOWN(f32),
    VOL(f32),
    EXIT,
    LOOP(bool),
    STATUS_PLAYING,
    STATUS_PAUSED,
}

pub struct Media {
    pub title: String,
    pub artist: String,
}
