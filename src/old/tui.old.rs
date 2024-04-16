use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, raw, start_color, Input, Window, COLOR_BLACK,
    COLOR_BLUE, COLOR_GREEN, COLOR_PAIR, COLOR_RED,
};
mod audio;
use audio::{play_sound, read_metadata, Command, Media};
use chrono::{DateTime, Duration, Local};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

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
    let keybinds: &str = "Press q to quit";
    let mut current_vol: f32 = 0.0;
    let mut is_paused: bool = false;
    tui_init_color();
    tui_additional_setup(&window);
    let mut media_metadata: Media = Media {
        title: "None".to_string(),
        artist: "None".to_string(),
    };
    let mut concated = concate_title_n_artist(&media_metadata);
    let (sender, receiver) = {
        let (s, r) = mpsc::channel::<Command>();
        (Arc::new(Mutex::new(s)), Arc::new(Mutex::new(r)))
    };
    let (sender2, receiver2) = {
        let (s, r) = mpsc::channel::<Command>();
        (Arc::new(Mutex::new(s)), Arc::new(Mutex::new(r)))
    };

    let mut clear_timer: DateTime<Local> = Local::now();

    loop {
        let max_x: i32 = window.get_max_x();
        let max_y: i32 = window.get_max_y();
        window.attron(COLOR_PAIR(3));
        window.mvprintw(max_y / 8, tui_get_str_center_x_coord(title, max_x), title);
        window.attroff(COLOR_PAIR(3));
        window.mvprintw(
            max_y - 1,
            tui_get_str_center_x_coord(keybinds, max_x),
            keybinds,
        );
        window.mvprintw(
            max_y - 2,
            tui_get_str_center_x_coord(&format!("Volume : {}", current_vol), max_x),
            format!("Volume : {}", current_vol),
        );

        let current_time: DateTime<Local> = Local::now();
        if current_time - clear_timer >= Duration::seconds(3) {
            window.clear();
            clear_timer = Local::now();
        }

        // to check if the thread is running or not
        let thread_status = *m_thread_status.lock().unwrap();
        if !thread_status {
            // if its not running then this
            let m_thread_status_clone = Arc::clone(&m_thread_status);
            let file_path_clone = file_path.clone();
            let receiver_clone = Arc::clone(&receiver);
            let sender2_clone = Arc::clone(&sender2);
            let receiver2_clone = Arc::clone(&receiver2);
            thread::spawn(move || {
                play_sound(file_path_clone, receiver_clone, sender2_clone).expect("ERR: Failed to play the audio.");
                *m_thread_status_clone.lock().unwrap() = false;
            });

            media_metadata = read_metadata(&file_path);
            concated = concate_title_n_artist(&media_metadata);
            *m_thread_status.lock().unwrap() = true;
        }
        window.attron(COLOR_PAIR(2));
        window.mvprintw(
            (max_y / 8) + 2,
            tui_get_str_center_x_coord(&concated, max_x),
            &concated,
        );
        let icon_l: &str;
        let icon_r: &str;
        if is_paused {
            icon_l = "|";
            icon_r = "|";
        } else {
            icon_l = ">";
            icon_r = "<";
        }
        window.mvprintw(
            (max_y / 8) + 2,
            tui_get_str_center_x_coord(&concated, max_x) - 2,
            icon_l,
        );
        window.mvprintw(
            (max_y / 8) + 2,
            tui_get_str_center_x_coord(&concated, max_x) + concated.len() as i32 + 1,
            icon_r,
        );
        window.attroff(COLOR_PAIR(2));

        match window.getch() {
            Some(Input::Character(c)) if c == 'q' || c == 'Q' => {
                sender.lock().unwrap().send(Command::STOP).unwrap();
                break;
            }
            Some(Input::Character(c)) if c == 'p' || c == 'P' || c == ' ' => {
                window.clear();
                sender.lock().unwrap().send(Command::PAUSED).unwrap();
                is_paused = !is_paused;
            }
            Some(Input::Character(c)) if c == '[' => {
                window.clear();
                sender.lock().unwrap().send(Command::VOLDOWN(0.05)).unwrap();
            }
            Some(Input::Character(c)) if c == ']' => {
                window.clear();
                sender.lock().unwrap().send(Command::VOLUP(0.05)).unwrap();
            }
            Some(Input::KeyResize) => {
                window.clear();
                ()
            }
            Some(_) => (),
            None => (),
        };

        match receiver2.lock().unwrap().try_recv() {
            Ok(Command::VOL(x)) => current_vol = x,
            Ok(_) => {}
            Err(_) => {}
        }
        window.mv(max_y + 1, max_x + 1);
        window.refresh();
    }
    endwin();
    return 0;
}
