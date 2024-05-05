use crate::{audio::read_metadata, command::*};
use std::sync::mpsc::{Receiver, Sender};
use chrono::{DateTime, Duration as CDuration, Local};
use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, raw, start_color, Input, Window, COLOR_BLACK,
    COLOR_BLUE, COLOR_GREEN, COLOR_PAIR, COLOR_RED,
};

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

pub fn start_window(path_to_file:String, receiver2_clone: &Receiver<Command>, sender_clone: &Sender<Command>) -> i8{
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

    media_metadata = read_metadata(&path_to_file);
    let mut concated = concate_title_n_artist(&media_metadata);
    sender_clone.send(Command::PLAY(path_to_file)).unwrap();

    // let mut clear_timer: DateTime<Local> = Local::now();

    loop {
        match receiver2_clone.try_recv() {
            Ok(Command::EXIT) => return 0,
            Ok(Command::VOL(v)) => {
                current_vol = v;
            },
            Ok(_) => (),
            Err(_) => (),
        }
        let percent_current_vol:i32 =(current_vol * 100.0) as i32; 

        window.erase();
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
            tui_get_str_center_x_coord(&format!("Volume : {}", percent_current_vol), max_x),
            format!("Volume : {}", percent_current_vol),
        );

        /* let current_time: DateTime<Local> = Local::now();
        if current_time - clear_timer >= CDuration::seconds(2) {
            window.clear();
            clear_timer = Local::now();
        } */

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
            Some(Input::KeyResize) => {
                window.clear();
            }
            Some(Input::Character(c)) if c == 'q' => {
                sender_clone.send(Command::EXIT).unwrap();
                break;
            }
            Some(Input::Character(c)) if c == 'p' || c == ' ' => {
                sender_clone.send(Command::PAUSED).unwrap();
                is_paused = !is_paused;
            }
            Some(Input::Character(c)) if c == '[' => {
                sender_clone.send(Command::VOLDOWN(0.05)).unwrap();
            }
            Some(Input::Character(c)) if c == ']' => {
                sender_clone.send(Command::VOLUP(0.05)).unwrap();
            }
            Some(_) => (),
            None => (),
        };

        window.mv(max_y + 1, max_x + 1);
        window.refresh();
    }
    endwin();
    return 0;
}
