// Colors based on 4th column, except for gray:
// https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg

use ansi_term::Color::Fixed;
use log::Level;
use std::env;
use std::path::Path;

fn paint(color: u8, value: &str) -> String {
    if no_color() || supports_color() < 2 {
        return value.to_owned();
    }

    Fixed(color).paint(value).to_string()
}

pub fn muted(value: &str) -> String {
    paint(238, value) // Gray
}

pub fn success(value: &str) -> String {
    paint(34, value) // Green
}

pub fn failure(value: &str) -> String {
    paint(161, value) // Red
}

pub fn invalid(value: &str) -> String {
    paint(185, value) // Yellow
}

pub fn path(path: &str) -> String {
    paint(36, path) // Teal
}

pub fn file_path(path: &Path) -> String {
    paint(38, &path.to_string_lossy()) // Turquoise
}

pub fn url(url: &str) -> String {
    paint(39, url) // Blue
}

pub fn shell(cmd: &str) -> String {
    paint(183, cmd) // Pink
}

pub fn symbol(value: &str) -> String {
    paint(111, value) // Purple
}

// Based on https://github.com/debug-js/debug/blob/master/src/common.js#L41
pub fn target(value: &str) -> String {
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
    let color = match level {
        Level::Error => 161, // Red
        Level::Warn => 185,  // Yellow
        Level::Info => 15,   // White
        Level::Debug => 45,  // Blue
        Level::Trace => 112, // Lime
    };

    paint(color, &level.as_str().to_lowercase())
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
