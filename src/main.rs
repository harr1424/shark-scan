use std::io;
use std::io::{Read};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use log::{info, LevelFilter};
use clap::{CommandFactory, FromArgMatches, Parser};
use serde::{Deserialize, Serialize};

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

fn parse_ports(port_arg: &str) -> Vec<u16> {
    let mut ports = Vec::new();
    for port in port_arg.split(',') {
        if port.contains(':') {
            let range: Vec<&str> = port.split(':').collect();
            if range.len() == 2 {
                let start: u16 = range[0].parse().expect("Invalid start port, expected similar to -p 1:1024");
                let end: u16 = range[1].parse().expect("Invalid end port, expected similar to -p 1:1024");
                for port in start..=end {
                    ports.push(port);
                }
            } else {
                panic!("Bad port range. Expected similar to -p 1:1024");
            }
        } else {
            let port: u16 = port.parse().expect(&format!("Invalid port: {}", port));
            ports.push(port);
        }
    }

    ports
}



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The target IP address to scan
    #[arg(short = 't', long)]
    target: String,
    /// The verbosity level (none, low, high)
    #[arg(short, long, default_value = "none")]
    verbosity: String,
    // The number of threads to use
    #[arg(short = 'n', long, default_value = "4")]
    threads: usize,
    /// The port range to scan in the format start:end or comma separated
    #[arg(short = 'p', long, default_value = "1:1024")]
    port_range: String
}

#[tokio::main]
async fn main() {
    let command = Args::command().arg_required_else_help(true);
    let matches = command.get_matches();
    let args = Args::from_arg_matches(&matches).expect("Failed to parse arguments");

    match args.verbosity.as_str() {
        "none" => env_logger::builder().filter_level(LevelFilter::Error).init(),
        "low" => env_logger::builder().filter_level(LevelFilter::Info).init(),
        "high" => env_logger::builder().filter_level(LevelFilter::Trace).init(),
        _ => env_logger::builder().filter_level(LevelFilter::Error).init(),
    }

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
