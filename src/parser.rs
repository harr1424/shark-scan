use clap::Parser;


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
    pub(crate) port_range: String
}

pub (crate) fn parse_ports(port_arg: &str) -> Vec<u16> {
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