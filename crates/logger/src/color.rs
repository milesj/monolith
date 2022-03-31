// Colors based on 4th column, except for gray:
// https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg

use ansi_term::Color::Fixed;
use log::Level;
use std::env;
use std::path::Path;

pub enum Color {
    White = 15,
    Black = 16,
    Green = 35,
    Teal = 36,
    Cyan = 38,
    Blue = 39,
    Purple = 111,
    Lime = 112,
    Red = 161,
    Pink = 183,
    Yellow = 185,
    Gray = 239,
    GrayLight = 248,
}

pub fn paint(color: u8, value: &str) -> String {
    if no_color() || supports_color() < 2 {
        return value.to_owned();
    }

    Fixed(color).paint(value).to_string()
}

pub fn muted(value: &str) -> String {
    paint(Color::Gray as u8, value)
}

pub fn muted_light(value: &str) -> String {
    paint(Color::GrayLight as u8, value)
}

pub fn success(value: &str) -> String {
    paint(Color::Green as u8, value)
}

pub fn failure(value: &str) -> String {
    paint(Color::Red as u8, value)
}

pub fn invalid(value: &str) -> String {
    paint(Color::Yellow as u8, value)
}

pub fn path(path: &str) -> String {
    paint(Color::Teal as u8, path)
}

pub fn file_path(path: &Path) -> String {
    paint(Color::Cyan as u8, path.to_str().unwrap_or("<unknown>"))
}

pub fn url(url: &str) -> String {
    paint(Color::Blue as u8, url)
}

pub fn shell(cmd: &str) -> String {
    paint(Color::Pink as u8, cmd)
}

pub fn symbol(value: &str) -> String {
    paint(Color::Lime as u8, value)
}

pub fn id(value: &str) -> String {
    paint(Color::Purple as u8, value)
}

pub fn target(value: &str) -> String {
    paint(Color::Blue as u8, value)
}

// Based on https://github.com/debug-js/debug/blob/master/src/common.js#L41
pub fn log_target(value: &str) -> String {
    if no_color() {
        return value.to_owned();
    }

    let mut hash: u32 = 0;

    for b in value.bytes() {
        hash = (hash << 5).wrapping_sub(hash) + b as u32;
    }

    // Lot of casting going on here...
    if supports_color() >= 2 {
        let index = i32::abs(hash as i32) as usize % COLOR_LIST.len();

        return Fixed(COLOR_LIST[index]).bold().paint(value).to_string();
    }

    let index = i32::abs(hash as i32) as usize % COLOR_LIST_UNSUPPORTED.len();

    Fixed(COLOR_LIST_UNSUPPORTED[index])
        .bold()
        .paint(value)
        .to_string()
}

pub fn log_level(level: Level) -> String {
    let mut msg = level.as_str().to_lowercase();

    while msg.len() < 5 {
        msg = [&msg, " "].concat();
    }

    match level {
        // Only color these as we want them to stand out
        Level::Error => paint(Color::Red as u8, &msg),
        Level::Warn => paint(Color::Yellow as u8, &msg),
        // Just return as is
        _ => msg,
        // Level::Info => Color::White,
        // Level::Debug => Color::Blue,
        // Level::Trace => Color::Lime,
    }
}

pub fn no_color() -> bool {
    env::var("NO_COLOR").is_ok()
}

// 0 = no
// 1 = 8
// 2 = 256
// 3 = 16m
pub fn supports_color() -> u8 {
    if no_color() {
        return 0;
    }

    if let Ok(var) = env::var("TERM") {
        if var == "dumb" {
            return 0;
        } else if var.contains("truecolor") {
            return 3;
        } else if var.contains("256") {
            return 2;
        }
    }

    if let Ok(var) = env::var("COLORTERM") {
        if var == "truecolor" || var == "24bit" {
            return 3;
        } else {
            return 1;
        }
    }

    if env::var("CI").is_ok() {
        return 2;
    }

    0
}

pub const COLOR_LIST: [u8; 76] = [
    20, 21, 26, 27, 32, 33, 38, 39, 40, 41, 42, 43, 44, 45, 56, 57, 62, 63, 68, 69, 74, 75, 76, 77,
    78, 79, 80, 81, 92, 93, 98, 99, 112, 113, 128, 129, 134, 135, 148, 149, 160, 161, 162, 163,
    164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 178, 179, 184, 185, 196, 197, 198, 199, 200,
    201, 202, 203, 204, 205, 206, 207, 208, 209, 214, 215, 220, 221,
];

pub const COLOR_LIST_UNSUPPORTED: [u8; 6] = [6, 2, 3, 4, 5, 1];
