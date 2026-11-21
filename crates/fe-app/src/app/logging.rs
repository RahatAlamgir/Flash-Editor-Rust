//! Writes log messages to `settings\flash-editor.log` (and the console in debug builds).

use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};

use super::paths;

/// Starts logging. Problems here are ignored: the app works without a log.
pub fn init() {
    let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();
    if cfg!(debug_assertions) {
        loggers.push(TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto));
    }
    let dir = paths::settings_dir();
    if std::fs::create_dir_all(&dir).is_ok()
        && let Ok(file) = std::fs::File::create(dir.join("flash-editor.log"))
    {
        loggers.push(WriteLogger::new(LevelFilter::Info, Config::default(), file));
    }
    let _ = CombinedLogger::init(loggers);
}
