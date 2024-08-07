//! # shark-scan
//!
//! `shark-scan` scans open ports on a target IP address. Users can
//! provide command line arguments to specify ports to scan as port ranges (1:1024),
//! comma separated lists (80,443), or both (80,443,1024:8080). Users can also specify
//! the number of threads to use when scanning, a timeout duration for connections in milliseconds,
//! output verbosity, and a probe option discussed in the Safety section below.
//!
//! # Safety
//! While this binary crate does not violate Rust's memory or type safety, executing this program
//! with the probe flag on an untrusted host may present a security risk.
//!
//! When the probe flag is provided, the following HTTP GET request will be sent to
//! open ports:
//!
//! ```ignore
//! let http_request = format!(
//!     "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
//!     target
//! );
//! match stream.write_all(http_request.as_bytes()).await {
//!     Ok(_) => info!("Sent HTTP GET request to {}", address),
//!     Err(e) => {
//!         error!("Failed to send HTTP GET request to {}: {:?}", address, e);
//!         return None;
//!     }
//! }
//!
//! let mut banner = vec![0; 1024];
//!
//! // Wait one full second to read response from server
//! match timeout(Duration::from_secs(1), stream.read(&mut banner)).await {
//!     Ok(Ok(n)) if n > 0 => {
//!         info!("Read {} bytes from {}", n, address);
//!         return Some(String::from_utf8_lossy(&banner[..n]).to_string());
//!     }
//!     Ok(Ok(_)) => {
//!         error!("No data read from {}", address);
//!     }
//!     Ok(Err(e)) => {
//!         error!("Failed to read from {}: {:?}", address, e);
//!     }
//!     Err(_) => {
//!         error!("Read operation timed out for {}", address);
//!     }
//! }
//! ```
//!
//! The decision to use this flag is left to the user. The author of this crate assumes 
//! no liability. 
//!
//!

pub mod scanner;
pub mod parser;