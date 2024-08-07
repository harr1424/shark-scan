use crate::parser::{parse_ports, Args};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};
use threadpool::ThreadPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::timeout;

/// A struct to contain scan results for a given port:
///
/// `status` will be set to open if a connection succeeds
///
/// If the --probe flag is used, `banner` will contain the first 1024 bytes
/// returned by the service on that port, if it supports HTTP
#[derive(Serialize, Deserialize)]
pub struct ScanResult {
    port: u16,
    status: String,
    banner: Option<String>,
}

async fn probe(target: &str, port: u16, timeout_ms: u64) -> Option<String> {
    let address = format!("{}:{}", target, port);
    let socket_addr: SocketAddr = address.to_socket_addrs().ok()?.next()?;

    info!("Attempting to connect to {}", address);

    match timeout(
        Duration::from_millis(timeout_ms),
        TcpStream::connect(&socket_addr),
    )
    .await
    {
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

            // Wait one full second to read response from server
            match timeout(Duration::from_secs(1), stream.read(&mut banner)).await {
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

async fn check_port(
    target: Arc<String>,
    port: u16,
    timeout_ms: u64,
    do_probe: bool,
    results: Arc<AsyncMutex<Vec<ScanResult>>>,
) {
    let address = format!("{}:{}", target, port);
    let socket_addr: SocketAddr = match address.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(addr) => addr,
            None => {
                error!("Could not resolve address: {}", address);
                return;
            }
        },
        Err(e) => {
            error!("Failed to resolve address {}: {:?}", address, e);
            return;
        }
    };

    match timeout(
        Duration::from_secs(timeout_ms),
        TcpStream::connect(&socket_addr),
    )
    .await
    {
        Ok(Ok(_)) => {
            if do_probe {
                let banner = probe(&target, port, timeout_ms).await;
                let mut results = results.lock().await;
                results.push(ScanResult {
                    port,
                    status: "open".to_string(),
                    banner,
                });
            } else {
                let mut results = results.lock().await;
                results.push(ScanResult {
                    port,
                    status: "open".to_string(),
                    banner: None,
                });
            }
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
        let probe = args.probe;
        pool.execute(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                check_port(target, port, timeout, probe, results).await;
            });
        });
    }

    pool.join();

    let end = Instant::now();
    let duration = end.duration_since(start);

    let results = results.lock();

    println!();
    for result in results.await.iter() {
        println!(
            "Port {} {}{}",
            result.port,
            result.status,
            result
                .banner
                .as_ref()
                .map(|b| format!(" - {}", b))
                .unwrap_or_default()
        );
    }

    println!(
        "\nScanning completed in {:.2} seconds",
        duration.as_secs_f64()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_open_port() {
        // Assuming the local network has port 80 open
        let target = Arc::new("192.168.1.1".to_string());
        let port = 80;
        let results = Arc::new(AsyncMutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        check_port(target, port, 100, false, results_clone).await;

        let results = results.lock().await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, port);
        assert_eq!(results[0].status, "open");
    }

    #[tokio::test]
    async fn test_closed_port() {
        // Assuming the local network has port 90 closed
        let target = Arc::new("192.168.1.1".to_string());
        let port = 90;
        let results = Arc::new(AsyncMutex::new(Vec::new()));
        let results_clone = Arc::clone(&results);

        check_port(target, port, 100, false, results_clone).await;

        let results = results.lock().await;
        assert_eq!(results.len(), 0);
    }
}
