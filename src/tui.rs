use crate::command::*;
use lofty::{Accessor, Probe, TaggedFileExt};
use std::sync::mpsc::{Receiver, Sender};
use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, raw, start_color, Input, Window, COLOR_BLACK,
    COLOR_BLUE, COLOR_GREEN, COLOR_PAIR, COLOR_RED,
};

fn read_metadata(path: &str) -> Media {
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

pub fn start_window(path_to_file:Vec<String>, receiver2_clone: &Receiver<Command>, sender_clone: &Sender<Command>) -> i8{
    let window = initscr();
    let title: &str = "[[ DUNGUS - Music Player ]]";
    let keybinds: &str = "Press q to quit";
    let mut current_vol: f32 = 0.0;
    let mut is_paused: bool = false;
    let mut is_loop: bool = false;
    tui_init_color();
    tui_additional_setup(&window);
    let mut media_metadata: Media;
    let mut concated_metadata: String;
    let mut counter: usize = 0;
    // true is next false is prev
    let mut counter_next: bool = true;

    media_metadata = read_metadata(&path_to_file[counter]);
    concated_metadata = concate_title_n_artist(&media_metadata);
    sender_clone.send(Command::PLAY(path_to_file[counter].clone())).unwrap();

    loop {
        let loop_icon: String;
        match is_loop {
            true => loop_icon = "L".to_string(),
            false => loop_icon = "l".to_string(),
        }
        match receiver2_clone.try_recv() {
            Ok(Command::EXIT) => {
                endwin();
                return 0;
            },
            Ok(Command::VOL(v)) => {
                current_vol = v;
            },
            Ok(Command::LOOP(v)) => {
               is_loop = v; 
            },
            Ok(Command::STOP) => {
                if is_loop {
                    sender_clone.send(Command::PLAY(path_to_file[counter].clone())).unwrap();
                } else if path_to_file.len() > 1 {
                    if counter_next {
                        counter += 1;
                    } else if counter == 0 {
                        counter = path_to_file.len() - 1;
                    } else {
                        counter -= 1;
                    }
                    media_metadata = read_metadata(&path_to_file[counter]);
                    concated_metadata = concate_title_n_artist(&media_metadata);
                    sender_clone.send(Command::PLAY(path_to_file[counter].clone())).unwrap();
                }
            }
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
            max_y - 3,
            tui_get_str_center_x_coord(&loop_icon, max_x),
            format!("{}", loop_icon),
        );
        window.mvprintw(
            max_y - 2,
            tui_get_str_center_x_coord(&format!("Volume : {}", percent_current_vol), max_x),
            format!("Volume : {}", percent_current_vol),
        );

        window.attron(COLOR_PAIR(2));
        window.mvprintw(
            (max_y / 8) + 2,
            tui_get_str_center_x_coord(&concated_metadata, max_x),
            &concated_metadata,
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
            tui_get_str_center_x_coord(&concated_metadata, max_x) - 2,
            icon_l,
        );
        window.mvprintw(
            (max_y / 8) + 2,
            tui_get_str_center_x_coord(&concated_metadata, max_x) + concated_metadata.len() as i32 + 1,
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
            Some(Input::Character(c)) if c == 'l' => {
                sender_clone.send(Command::LOOP(!is_loop)).unwrap();
            }
            Some(Input::Character(c)) if c == 'j' => {
                sender_clone.send(Command::STOP).unwrap();
                counter_next = false;
            }
            Some(Input::Character(c)) if c == 'k' => {
                sender_clone.send(Command::STOP).unwrap();
                counter_next = true;
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
