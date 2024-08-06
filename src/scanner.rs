use std::io;
use std::io::{Read};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use log::{info};
use serde::{Deserialize, Serialize};

use crate::{parser::{parse_ports, Args}};
#[derive(Serialize, Deserialize)]
struct ScanResult {
    port: u16,
    status: String,
    banner: Option<String>,
}

fn grab_banner(target: &str, port: u16) -> Option<String> {
    let address = format!("{}:{}", target, port);
    if let Ok(mut stream) = TcpStream::connect(&address) {
        let mut banner = [0; 1024];
        if let Ok(_) = stream.read(&mut banner) {
            return Some(String::from_utf8_lossy(&banner).to_string());
        }
    }
    None
}

fn check_port(target: Arc<String>, port: u16, results: Arc<Mutex<Vec<ScanResult>>>) {
    let address = format!("{}:{}", target, port);
    match TcpStream::connect_timeout(&address.parse().unwrap(), Duration::from_secs(1)) {
        Ok(_) => {
            let banner = grab_banner(&target, port);
            let mut results = results.lock().unwrap();
            results.push(ScanResult {
                port,
                status: "open".to_string(),
                banner,
            });
        }

        Err(e) => {
            let status = match e.kind() {
                io::ErrorKind::TimedOut => "timed out",
                io::ErrorKind::ConnectionRefused => "refused",
                _ => "failed",
            };
            info!("Port {} {}", port, status);
        }
    }
}

pub async fn scan(args: Args) {
    let ports = parse_ports(&args.port_range);
    let target = Arc::new(args.target.trim().to_string());

    println!("{}", "*".repeat(40));
    println!("* Scanning: {} *", target);
    println!("{}", "*".repeat(40));

    let start = Instant::now();

    let pool = ThreadPool::new(args.threads);
    let results = Arc::new(Mutex::new(Vec::new()));

    for port in ports {
        let results = Arc::clone(&results);
        let target = Arc::clone(&target);
        pool.execute(move || {
            check_port(target, port, results);
        });
    }

    pool.join();

    let end = Instant::now();
    let duration = end.duration_since(start);

    let results = results.lock().unwrap();
    for result in results.iter() {
        println!("Port {} {}{}", result.port, result.status, result.banner.as_ref().map(|b| format!(" - {}", b)).unwrap_or_default());
    }

    println!("\nScanning completed in {:.2} seconds", duration.as_secs_f64());
}