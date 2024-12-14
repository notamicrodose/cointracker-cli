use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use anyhow::Result;

pub fn log_error(category: &str, message: &str) -> Result<()> {
    log_message("ERROR", category, message)
}

pub fn log_info(category: &str, message: &str) -> Result<()> {
    log_message("INFO", category, message)
}

fn log_message(level: &str, category: &str, message: &str) -> Result<()> {
    let now = Local::now();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("crypto_tracker.log")?;

    writeln!(
        file,
        "[{}] {} - {}: {}", 
        now.format("%Y-%m-%d %H:%M:%S"),
        level,
        category,
        message
    )?;

    Ok(())
}
