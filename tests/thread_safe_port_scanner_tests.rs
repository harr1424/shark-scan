
#[cfg(test)]
mod tests {
    use tokio::sync::Mutex as AsyncMutex;
    use std::sync::Arc;
    use thread_safe_port_scanner::parser::parse_ports;
    use thread_safe_port_scanner::scanner::check_port;

    #[tokio::test]
    async fn test_parse_ports() {
        let port_range = "20-25";
        let ports = parse_ports(port_range);
        assert_eq!(ports, vec![20, 21, 22, 23, 24, 25]);
    }

    #[tokio::test]
    async fn test_check_open_port() {
        // Assuming the local machine has port 80 open (HTTP)
        let target = Arc::new("127.0.0.1".to_string());
        let port = 80;
        let results = Arc::new(AsyncMutex::new(Vec::new()));

        check_port(target, port, results.clone()).await;

        let results = results.lock().await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, port);
        assert_eq!(results[0].status, "open");
    }

    #[tokio::test]
    async fn test_check_closed_port() {
        // Assuming the local machine has port 81 closed
        let target = Arc::new("127.0.0.1".to_string());
        let port = 81;
        let results = Arc::new(AsyncMutex::new(Vec::new()));

        check_port(target, port, results.clone()).await;

        let results = results.lock().await;
        // It won't be pushed to results if it's closed
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_grab_banner() {
        // Assuming a simple TCP server running on localhost:12345 that sends "Hello"
        let target = "127.0.0.1";
        let port = 12345;

        if let Some(banner) = grab_banner(target, port).await {
            assert!(banner.contains("Hello"));
        } else {
            panic!("Failed to grab banner");
        }
    }
}
