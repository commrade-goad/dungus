use pancurses::*;
mod audio;
use audio::{play_sound, Media, read_metadata};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

enum Command {
    PAUSED,
}

fn tui_init_color() {
    start_color();
    init_pair(1, COLOR_RED, COLOR_BLACK);
    init_pair(2, COLOR_BLUE, COLOR_BLACK);
    init_pair(3, COLOR_GREEN, COLOR_BLACK);
}

fn tui_additional_setup(win: &Window) {
    curs_set(0);
    noecho();
    raw();
    win.nodelay(true);
    win.timeout(0);
    win.keypad(true);
}

fn tui_get_str_center_x_coord(str: &str, x: i32) -> i32 {
    return (x / 2) - (str.len() as i32 / 2);
}

fn concate_title_n_artist(metadata: &Media) -> String {
    return format!("{} - {}", metadata.title, metadata.artist);
}

pub fn start_window(file: &str) -> i8 {
    let m_thread_status: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let file_path: String = file.to_string();
    let window = initscr();
    let title: &str = "[[ DUNGUS - Music Player ]]";
    let quit: &str = "Press q or Q to quit";
    tui_init_color();
    tui_additional_setup(&window);
    let mut media_metadata: Media = Media {
        title: "None".to_string(),
        artist: "None".to_string(),
    };
    let mut concated = concate_title_n_artist(&media_metadata);
    let (sender, receiver) = mpsc::channel::<Command>();

    loop {
        let max_x: i32 = window.get_max_x();
        let max_y: i32 = window.get_max_y();
        window.attron(COLOR_PAIR(3));
        window.mvprintw(max_y / 8, tui_get_str_center_x_coord(title, max_x), title);
        window.attroff(COLOR_PAIR(3));
        window.mvprintw(max_y - 1, tui_get_str_center_x_coord(quit, max_x), quit);

        // to check if the thread is running or not
        let thread_status = *m_thread_status.lock().unwrap();
        if !thread_status { // if its not running then this
            let m_thread_status_clone = Arc::clone(&m_thread_status);
            let file_path_clone = file_path.clone();
            thread::spawn(move || {
                let sink:rodio::Sink = play_sound(file_path_clone).expect("ERR: Failed to play the audio.");
                *m_thread_status_clone.lock().unwrap() = false;
                // sink.sleep_until_end();
            });

            media_metadata = read_metadata(&file_path);
            concated = concate_title_n_artist(&media_metadata);
            *m_thread_status.lock().unwrap() = true;
        }

        window.attron(COLOR_PAIR(2));
        window.mvprintw((max_y / 3) - 2, tui_get_str_center_x_coord(&concated, max_x), &concated);
        window.attroff(COLOR_PAIR(2));

        match window.getch() {
            Some(Input::Character(c)) if c == 'q' || c == 'Q' => break,
            Some(Input::KeyResize) => window.clear(),
            Some(_) => 0,
            None => 0,
        };
        window.refresh();
    }
    endwin();
    return 0;
}
