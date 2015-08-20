use log::{self, Log, LogRecord, LogMetadata, LogLevel, LogLevelFilter, SetLoggerError};


macro_rules! stylish {
    ($data:expr, $style:expr) => (concat!("\x1b[", $style, "m", $data, "\x1b[0m"))
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, _: &LogMetadata) -> bool { true }

    fn log(&self, record: &LogRecord) {
        let marker = match record.level() {
            LogLevel::Error => stylish!("•", "31"),
            LogLevel::Warn  => stylish!("•", "33"),
            LogLevel::Info  => stylish!("•", "34"),
            LogLevel::Debug => stylish!("•", "35"),
            LogLevel::Trace => stylish!("•", "37")
        };

        let node = record.location().module_path().rsplit_terminator(':').next().unwrap();
        let message = record.args();

        println!("{} {:10} | {}", marker, node, message);
    }
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Info);
        Box::new(Logger)
    })
}
