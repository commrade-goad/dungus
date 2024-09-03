use crate::command::*;
use lofty::{Accessor, Probe, TaggedFileExt};
use std::sync::mpsc::{Receiver, Sender};
use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, raw, start_color, Attribute, Input, Window, COLOR_BLACK, COLOR_BLUE, COLOR_GREEN, COLOR_PAIR, COLOR_RED, COLOR_WHITE
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
    init_pair(4, COLOR_WHITE, COLOR_BLACK);
}

fn tui_additional_setup(win: &Window) {
    curs_set(0);
    noecho();
    raw();
    win.nodelay(true);
    win.timeout(0);
    win.keypad(true);
}

fn is_multibyte(s: &str) -> bool {
    s.chars().any(|c| {c.is_cjk()})
}

fn tui_get_str_center_x_coord(str: &str, x: i32) -> i32 {
    if is_multibyte(str) {
        return (x/2) - (str.chars().count() as i32 / 2);
    }
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
    let mut media_metadata: Vec<Media> = Vec::new();
    let mut concated_metadata: Vec<String> = Vec::new();
    let mut counter: usize = 0;
    // true is next false is prev
    let mut counter_next: bool = true;

    for i in 0..path_to_file.len() {
        media_metadata.push(read_metadata(&path_to_file[i]));
        concated_metadata.push(concate_title_n_artist(&media_metadata[i]));
    }
    sender_clone.send(Command::PLAY(path_to_file[counter].clone())).unwrap();

    loop {
        let loop_icon: String;
        match is_loop {
            true => loop_icon = "Loop".to_string(),
            false => loop_icon = "____".to_string(),
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
                    if counter_next && counter < path_to_file.len() - 1 {
                        counter += 1;
                    } else if counter <= 0 {
                        counter = path_to_file.len() - 1;
                    } else if !counter_next {
                        counter -= 1;
                    } else {
                        counter = 0;
                    }
                    sender_clone.send(Command::PLAY(path_to_file[counter].clone())).unwrap();
                }
            }
            Ok(Command::STATUS_PAUSED) => {
                is_paused = true;
            }
            Ok(Command::STATUS_PLAYING) => {
                is_paused = false;
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
            max_y - 1,
            1,
            format!("[{}]", loop_icon),
        );
        window.mvprintw(
            max_y - 2,
            tui_get_str_center_x_coord(&format!("Volume : {}", percent_current_vol), max_x),
            format!("Volume : {}", percent_current_vol),
        );

        window.attron(COLOR_PAIR(2));
        window.attron(Attribute::Bold);
        window.mvprintw(
            (max_y / 8) + 2,
            tui_get_str_center_x_coord(&concated_metadata[counter], max_x),
            &concated_metadata[counter],
        );
        window.attroff(Attribute::Bold);

        let mut print_at: i32 = 1;
        for i in 0..media_metadata.len() {
            if i != counter {
                window.mvprintw(
                    (max_y / 8 as i32) + 3 + print_at as i32,
                    tui_get_str_center_x_coord(&concated_metadata[i], max_x),
                    &concated_metadata[i],
                );
                if i == counter + 1 {
                    window.mvprintw(
                        (max_y / 8 as i32) + 3 + print_at as i32,
                        tui_get_str_center_x_coord(&concated_metadata[i], max_x) - 2,
                        "N",
                    );
                }
            } else {
                window.attron(COLOR_PAIR(4));
                window.mvprintw(
                    (max_y / 8 as i32) + 3 + print_at as i32,
                    tui_get_str_center_x_coord(&concated_metadata[i], max_x),
                    &concated_metadata[i],
                );
                window.attron(COLOR_PAIR(2));
            }
            print_at += 1;
        }

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
            tui_get_str_center_x_coord(&concated_metadata[counter], max_x) - 2,
            icon_l,
        );
        window.mvprintw(
            (max_y / 8) + 2,
            tui_get_str_center_x_coord(&concated_metadata[counter], max_x) + concated_metadata[counter].len() as i32 + 1,
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
            Some(Input::Character(c)) if c == 'k' => {
                sender_clone.send(Command::STOP).unwrap();
                counter_next = false;
            }
            Some(Input::Character(c)) if c == 'j' => {
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

trait IsCJK {
    fn is_cjk(&self) -> bool;
}

impl IsCJK for char {
    fn is_cjk(&self) -> bool {
        // Unicode ranges for CJK characters
        let cjk_ranges = [
            (0x4E00, 0x9FFF),  // CJK Unified Ideographs
            (0x3400, 0x4DBF),  // CJK Unified Ideographs Extension A
            (0x20000, 0x2A6DF), // CJK Unified Ideographs Extension B
            (0x2A700, 0x2B73F), // CJK Unified Ideographs Extension C
            (0x2B740, 0x2B81F), // CJK Unified Ideographs Extension D
            (0x2B820, 0x2CEAF), // CJK Unified Ideographs Extension E
            (0xF900, 0xFAFF),  // CJK Compatibility Ideographs
            (0x2F800, 0x2FA1F), // CJK Compatibility Ideographs Supplement
        ];
        let code_point = *self as u32;
        cjk_ranges.iter().any(|&(start, end)| code_point >= start && code_point <= end)
    }
}
