mod cert;
mod fs;
mod http;
mod https;
mod log;

use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub root: PathBuf,
    pub port: u16,
    pub https: bool,
    pub ignore_index: bool,
}

fn usage() {
    println!("serve 1.0.0");
    println!();
    println!("USAGE:");
    println!("  serve <path> [port] [--http|--https] [-i]");
    println!();
    println!("FLAGS:");
    println!("  --http      Use HTTP");
    println!("  --https     Use HTTPS");
    println!("  -i          Ignore index.html");
    println!("  --help      Show help");
    println!("  --version   Show version");
    println!();
    println!("DEFAULTS:");
    println!("  protocol = http");
    println!("  port = 8000");
    println!();
    println!("EXAMPLES:");
    println!("  serve .");
    println!("  serve . 3000");
    println!("  serve . 8443 --https");
    println!("  serve . -i");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        usage();
        return;
    }

    if args.iter().any(|a| a == "--help") {
        usage();
        return;
    }

    if args.iter().any(|a| a == "--version") {
        println!("serve 1.0.0");
        return;
    }

    let mut path = None;
    let mut port = 8000u16;
    let mut https = false;
    let mut ignore_index = false;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--https" => https = true,
            "--http" => https = false,
            "-i" => ignore_index = true,
            _ => {
                if path.is_none() {
                    path = Some(PathBuf::from(arg));
                } else if let Ok(p) = arg.parse::<u16>() {
                    port = p;
                }
            }
        }
    }

    let root = match path {
        Some(p) => p,
        None => {
            usage();
            return;
        }
    };

    if !root.exists() || !root.is_dir() {
        log::error("invalid directory");
        return;
    }

    let cfg = Config {
        root,
        port,
        https,
        ignore_index,
    };

    if cfg.https {
        https::run(cfg).await;
    } else {
        http::run(cfg).await;
    }
}
