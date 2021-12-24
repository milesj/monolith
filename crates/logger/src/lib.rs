use chrono::prelude::*;
use chrono::Local;
use fern::Dispatch;
use log::LevelFilter;
use std::io;

pub mod color;

static mut FIRST_LOG: bool = true;
static mut LAST_HOUR: u32 = 0;

pub struct Logger {}

impl Logger {
    pub fn init(level: LevelFilter) {
        if level == LevelFilter::Off {
            return;
        }

        let mut dispatch = Dispatch::new()
            .format(|out, message, record| {
                let mut date_format = "[%Y-%m-%d %H:%M:%S]";
                let current_timestamp = Local::now();

                // Shorten the timestamp when within the same hour
                unsafe {
                    if !FIRST_LOG && current_timestamp.hour() == LAST_HOUR {
                        date_format = "[%H:%M:%S]";
                    }

                    FIRST_LOG = false;
                    LAST_HOUR = current_timestamp.hour();
                }

                // We dont need the label when level is "info"
                if record.level() == log::Level::Info {
                    out.finish(format_args!(
                        "{} {} {}",
                        color::muted(&current_timestamp.format(date_format).to_string()),
                        color::target(record.target()),
                        message
                    ));
                } else {
                    out.finish(format_args!(
                        "{} {} {} {}",
                        color::muted(&current_timestamp.format(date_format).to_string()),
                        color::target(record.target()),
                        color::log_level(record.level()),
                        message
                    ));
                }
            })
            // Pipe errors to stderr
            .chain(
                Dispatch::new()
                    .level(LevelFilter::Error)
                    .chain(io::stderr()),
            );

        // All other log types go to stdout
        if level != LevelFilter::Error {
            dispatch = dispatch.chain(fern::Dispatch::new().level(level).chain(io::stdout()));
        }

        dispatch.apply().unwrap();
    }
}
