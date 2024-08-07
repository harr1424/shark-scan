## shark-scan

Allows for fearless concurrent scanning of open TCP ports on a target IP address:

```text
shark-scan
An async multi-threaded port scanner supporting user specified port ranges, timeout durations, and thread quantity

Usage: shark-scan [OPTIONS] --target <TARGET>

Options:
  -t, --target <TARGET>          The target IP address to scan
  -v, --verbosity <VERBOSITY>    The verbosity level (none, low, high) [default: none]
  -n, --threads <THREADS>        [default: 4]
  -p, --port-range <PORT_RANGE>  The port range to scan in the format start:end or comma separated [default: 1:1024]
  -m, --timeout <TIMEOUT>        The time in milliseconds to await successful port connection [default: 100]
      --probe                    ***Do not use against untrusted hosts*** Probe the socket by performing an HTTP GET request
  -h, --help                     Print help
  -V, --version                  Print version
```

### Examples
```text
shark-scan -t 192.168.1.1 -n 6 
****************************************
* Scanning: 192.168.1.1 *
****************************************

Port 53 open
Port 80 open
Port 443 open

Scanning completed in 0.40 seconds
```

```text
shark-scan -t 192.168.1.1 -p 20:25 -n 6 -m 1000 -v high
****************************************
* Scanning: 192.168.1.1 *
****************************************
[2024-08-07T09:12:47Z INFO  shark_scan::scanner] Port 24 refused
[2024-08-07T09:12:47Z INFO  shark_scan::scanner] Port 21 refused
[2024-08-07T09:12:47Z INFO  shark_scan::scanner] Port 22 refused
[2024-08-07T09:12:47Z INFO  shark_scan::scanner] Port 25 refused
[2024-08-07T09:12:47Z INFO  shark_scan::scanner] Port 20 refused
[2024-08-07T09:12:47Z INFO  shark_scan::scanner] Port 23 refused


Scanning completed in 0.00 seconds
```

### Safety
While this binary crate does not violate Rust's memory or type safety, executing this program
with the `--probe` flag on an untrusted host may present a security risk. When this flag is used, 
the following code will execute: 
```rust
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
```

A malicious host may respond with a payload crafted to damage your system. 
If the `--probe` flag is not supplied, no HTTP requests will be sent, and the program will simply 
try to complete a TCP connection with the target IP address ports. The decision to use this feature is left to 
the crate's user and the author assumes no liability for any consequences. 

### Probe functionality 
Currently the functionality provided by passing the `--probe` flag is limited: it will only perform an HTTP GET 
request to the service root ("/") endpoint. In the future, I would like to research additional probes that work 
on services not supporting HTTP. An example of current functionality is shown below: 

```text
shark-scan -t 192.168.1.1 -n 6 --probe
****************************************
* Scanning: 192.168.1.1 *
****************************************
[2024-08-07T09:07:43Z ERROR shark_scan::scanner] No data read from 192.168.1.1:443
[2024-08-07T09:07:44Z ERROR shark_scan::scanner] Read operation timed out for 192.168.1.1:53

Port 80 open - HTTP/1.1 308 Permanent Redirect
Location: https://192.168.1.1/
Content-Length: 0
Connection: close
Date: Wed, 07 Aug 2024 09:07:28 GMT
Server: lighttpd/1.4.59


Port 443 open
Port 53 open

Scanning completed in 1.03 seconds
```

Another example: 

```text
shark-scan -t  104.21.94.80 -p 80,443 --probe
****************************************
* Scanning: 104.21.94.80 *
****************************************

Port 443 open - HTTP/1.1 400 Bad Request
Server: cloudflare
Date: Wed, 07 Aug 2024 09:08:42 GMT
Content-Type: text/html
Content-Length: 253
Connection: close
CF-RAY: -

<html>
<head><title>400 The plain HTTP request was sent to HTTPS port</title></head>
<body>
<center><h1>400 Bad Request</h1></center>
<center>The plain HTTP request was sent to HTTPS port</center>
<hr><center>cloudflare</center>
</body>
</html>

Port 80 open - HTTP/1.1 403 Forbidden
Date: Wed, 07 Aug 2024 09:08:43 GMT
Content-Type: text/plain; charset=UTF-8
Content-Length: 16
Connection: close
X-Frame-Options: SAMEORIGIN
Referrer-Policy: same-origin
Cache-Control: private, max-age=0, no-store, no-cache, must-revalidate, post-check=0, pre-check=0
Expires: Thu, 01 Jan 1970 00:00:01 GMT
Server: cloudflare
CF-RAY: 8af61faa6fd130d1-SEA

error code: 1003

Scanning completed in 0.43 seconds
```

As you can see, some useful information might be obtained. Pull requests adding to this functionality are welcome. 

