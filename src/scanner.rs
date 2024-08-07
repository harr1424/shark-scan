use std::io;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::sync::Mutex as AsyncMutex;
use std::time::{Duration, Instant};
use std::sync::{Arc};
use threadpool::ThreadPool;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::{parser::{parse_ports, Args}};
#[derive(Serialize, Deserialize)]
pub struct ScanResult {
    port: u16,
    status: String,
    banner: Option<String>,
}

///
pub async fn grab_banner(target: &str, port: u16, timeout_secs: u64) -> Option<String> {
    let address = format!("{}:{}", target, port);
    info!("Attempting to connect to {}", address);

    // Set a timeout for the connection attempt
    match timeout(Duration::from_secs(timeout_secs), TcpStream::connect(&address)).await {
        Ok(Ok(mut stream)) => {
            info!("Connected to {}", address);

            let http_request = format!(
                "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                target
            );
            match stream.write_all(http_request.as_bytes()).await {
                Ok(_) => info!("Sent HTTP GET request to {}", address),
                Err(e) => {
                    error!("Failed to send HTTP GET request to {}: {:?}", address, e);
                    return None;
                }
            }

            let mut banner = vec![0; 1024];

            // Set a timeout for the read operation
            match timeout(Duration::from_secs(5), stream.read(&mut banner)).await {
                Ok(Ok(n)) if n > 0 => {
                    info!("Read {} bytes from {}", n, address);
                    return Some(String::from_utf8_lossy(&banner[..n]).to_string());
                }
                Ok(Ok(_)) => {
                    error!("No data read from {}", address);
                }
                Ok(Err(e)) => {
                    error!("Failed to read from {}: {:?}", address, e);
                }
                Err(_) => {
                    error!("Read operation timed out for {}", address);
                }
            }
        }
        Ok(Err(e)) => {
            error!("Failed to connect to {}: {:?}", address, e);
        }
        Err(_) => {
            error!("Connection attempt timed out for {}", address);
        }
    }

    None
}

pub async fn check_port(target: Arc<String>, port: u16, timeout_secs: u64, results: Arc<AsyncMutex<Vec<ScanResult>>>) {
    let address = format!("{}:{}", target, port);
    match timeout(Duration::from_secs(timeout_secs), TcpStream::connect(&address)).await {
        Ok(Ok(_)) => {
            let banner = grab_banner(&target, port, timeout_secs).await;
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
        let timeout = args.timeout;
        pool.execute(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                check_port(target, port, timeout, results).await;
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