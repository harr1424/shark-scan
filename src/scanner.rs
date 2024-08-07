use std::io;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::sync::Mutex as AsyncMutex;
use std::time::{Duration, Instant};
use std::sync::{Arc};
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

async fn grab_banner(target: &str, port: u16) -> Option<String> {
    let address = format!("{}:{}", target, port);
    if let Ok(stream) = TcpStream::connect(&address).await {
        let mut banner = [0; 1024];
        if let Ok(_) = stream.try_read(&mut banner) {
            return Some(String::from_utf8_lossy(&banner).to_string());
        }
    }
    None
}

async fn check_port(target: Arc<String>, port: u16, results: Arc<AsyncMutex<Vec<ScanResult>>>) {
    let address = format!("{}:{}", target, port);
    match timeout(Duration::from_secs(1), TcpStream::connect(&address)).await {
        Ok(Ok(_)) => {
            let banner = grab_banner(&target, port).await;
            let mut results = results.lock().await;
            results.push(ScanResult {
                port,
                status: "open".to_string(),
                banner,
            });
        }
        Ok(Err(e)) => {
            let status = match e.kind() {
                io::ErrorKind::ConnectionRefused => "refused",
                _ => "failed",
            };
            info!("Port {} {}", port, status);
        }
        Err(_) => {
            info!("Port {} timed out", port);
        }
    }
}

/// Parses target IP address and port range(S) from args and uses the specified number
/// of threads to create a threadpool in order to scan open ports concurrently across threads
pub async fn scan(args: Args) {
    let ports = parse_ports(&args.port_range);
    let target = Arc::new(args.target.trim().to_string());

    println!("{}", "*".repeat(40));
    println!("* Scanning: {} *", target);
    println!("{}", "*".repeat(40));

    let start = Instant::now();

    let pool = ThreadPool::new(args.threads);
    let results = Arc::new(AsyncMutex::new(Vec::new()));

    for port in ports {
        let results = Arc::clone(&results);
        let target = Arc::clone(&target);
        pool.execute(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                check_port(target, port, results).await;
            });
        });
    }

    pool.join();

    let end = Instant::now();
    let duration = end.duration_since(start);

    let results = results.lock();
    for result in results.await.iter() {
        println!("Port {} {}{}", result.port, result.status, result.banner.as_ref().map(|b| format!(" - {}", b)).unwrap_or_default());
    }

    println!("\nScanning completed in {:.2} seconds", duration.as_secs_f64());
}