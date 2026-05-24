use crate::Config;
use chrono::Local;
use colored::*;

fn ts() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

pub fn startup(cfg: &Config) {
    println!(
        "{} {} {} {} {}",
        format!("[{}]", ts()).bright_black(),
        "SERVE".bright_green().bold(),
        if cfg.https {
            "HTTPS".bright_magenta().bold()
        } else {
            "HTTP".bright_cyan().bold()
        },
        format!("PORT {}", cfg.port).bright_yellow().bold(),
        cfg.root.display().to_string().bright_white()
    );
}

pub fn request(method: &str, path: &str, code: u16, ip: &str) {
    let status = match code {
        200..=299 => code.to_string().green(),
        300..=399 => code.to_string().yellow(),
        400..=499 => code.to_string().red(),
        _ => code.to_string().bright_red(),
    };

    println!(
        "{} {} {} {} {}",
        format!("[{}]", ts()).bright_black(),
        method.bright_blue().bold(),
        status.bold(),
        ip.bright_magenta(),
        path.white()
    );
}

pub fn served(path: &str, size: u64) {
    println!(
        "{} {} {} {}",
        format!("[{}]", ts()).bright_black(),
        "FILE".bright_green().bold(),
        format!("{} bytes", size).bright_cyan(),
        path.bright_white()
    );
}

pub fn listing(path: &str, dirs: usize, files: usize) {
    println!(
        "{} {} {} {} {}",
        format!("[{}]", ts()).bright_black(),
        "LIST".bright_yellow().bold(),
        format!("dirs={}", dirs).bright_blue(),
        format!("files={}", files).bright_magenta(),
        path.white()
    );
}

pub fn cert(path: &str) {
    println!(
        "{} {} {}",
        format!("[{}]", ts()).bright_black(),
        "CERT".bright_magenta().bold(),
        path.white()
    );
}

pub fn error(msg: &str) {
    eprintln!(
        "{} {} {}",
        format!("[{}]", ts()).bright_black(),
        "ERROR".bright_red().bold(),
        msg.red()
    );
}
