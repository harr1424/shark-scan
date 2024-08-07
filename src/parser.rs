use clap::Parser;

/// The Args struct is used to contain and parse command line arguments
///
/// -t --target <target ip address> The IP address to scan
///
/// -v --verbosity <[none, low, high]> The level of program output
///
/// -n --threads <int> The number of threads to use
///
/// -p --port-range <int:int,int,int> The port range or comma separated ports to scan
///
/// -m --timeout <int> The number of milliseconds to wait for a connection on a given port
///
/// --probe When this is provided an HTTP GET request will be sent to the port
///
/// Do not use --probe when scanning untrusted hosts as they may send a malicious response
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The target IP address to scan
    #[arg(short = 't', long)]
    pub(crate) target: String,
    /// The verbosity level (none, low, high)
    #[arg(short, long, default_value = "none")]
    pub verbosity: String,
    // The number of threads to use
    #[arg(short = 'n', long, default_value = "4")]
    pub(crate) threads: usize,
    /// The port range to scan in the format start:end or comma separated
    #[arg(short = 'p', long, default_value = "1:1024")]
    pub(crate) port_range: String,
    /// The time in milliseconds to await successful port connection
    #[arg(short = 'm', long, default_value = "100")]
    pub(crate) timeout: u64,
    /// ***Do not use against untrusted hosts***
    /// Probe the socket by performing an HTTP GET request
    #[arg(long)]
    pub(crate) probe: bool,
}

pub(crate) fn parse_ports(port_arg: &str) -> Vec<u16> {
    let mut ports = Vec::new();
    for port in port_arg.split(',') {
        let port = port.trim();
        if port.contains(':') {
            let range: Vec<&str> = port.split(':').collect();
            if range.len() == 2 {
                let start: u16 = range[0]
                    .parse()
                    .expect("Invalid start port, expected similar to -p 1:1024");
                let end: u16 = range[1]
                    .parse()
                    .expect("Invalid end port, expected similar to -p 1:1024");
                for port in start..=end {
                    ports.push(port);
                }
            } else {
                panic!("Invalid port: expected similar to -p 1:1024");
            }
        } else {
            let port: u16 = port.parse().expect(&format!("Invalid port: {}", port));
            ports.push(port);
        }
    }

    ports
}

#[cfg(test)]
mod tests {
    use super::parse_ports;
    #[test]
    fn test_parse_ports() {
        let port_range = "20:25,31,32,45:50";
        let ports = parse_ports(port_range);
        assert_eq!(
            ports,
            vec![20, 21, 22, 23, 24, 25, 31, 32, 45, 46, 47, 48, 49, 50]
        );
    }

    #[test]
    fn test_parse_ports_list_trimmed() {
        let port_range = "14, 15, 29";
        let ports = parse_ports(port_range);
        assert_eq!(ports, vec![14, 15, 29]);
    }

    #[test]
    #[should_panic(expected = "Invalid port:")]
    fn test_parse_invalid_port_range() {
        let port_range = "14-15";
        let _ = parse_ports(port_range);
    }
    #[test]
    #[should_panic(expected = "Invalid port:")]
    fn test_parse_invalid_port_value() {
        let port_range = "14, a2";
        let _ = parse_ports(port_range);
    }
}
