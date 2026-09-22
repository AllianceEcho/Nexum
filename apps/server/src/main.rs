//! Nexum TCP server with configuration support.

use nexum_core::Core;
use nexum_protocol::{Credential, RpcDispatcher, parse_request, serialize_response};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Server configuration loaded from a simple config file or defaults.
#[derive(Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub max_connections: usize,
    pub data_dir: PathBuf,
    pub require_auth: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 39100,
            max_connections: 100,
            data_dir: PathBuf::from("./data"),
            require_auth: false,
        }
    }
}

impl ServerConfig {
    /// Parse a simple key=value config file, falling back to defaults for missing fields.
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read config file {path}: {e}"))?;
        let mut port = None;
        let mut max_connections = None;
        let mut data_dir = None;
        let mut require_auth = None;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "port" => {
                        if let Ok(p) = value.trim().parse::<u16>() {
                            port = Some(p);
                        }
                    }
                    "max_connections" => {
                        if let Ok(m) = value.trim().parse::<usize>() {
                            max_connections = Some(m);
                        }
                    }
                    "data_dir" => {
                        data_dir = Some(PathBuf::from(value.trim()));
                    }
                    "require_auth" => {
                        require_auth = Some(value.trim().parse::<bool>().unwrap_or(false));
                    }
                    _ => {}
                }
            }
        }
        Ok(Self {
            port: port.unwrap_or(Self::default().port),
            max_connections: max_connections.unwrap_or(Self::default().max_connections),
            data_dir: data_dir.unwrap_or(Self::default().data_dir),
            require_auth: require_auth.unwrap_or(Self::default().require_auth),
        })
    }
}

fn handle_connection(mut stream: TcpStream, core: Arc<Mutex<Core>>) -> std::io::Result<()> {
    let reader_stream = stream.try_clone()?;
    let reader = BufReader::new(reader_stream);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Log credential scheme for debugging (if present and not None)
        if let Ok(request) = parse_request(&line)
            && let Some(ref cred) = request.credential
        {
            match cred {
                Credential::None => {}
                Credential::Bearer { .. } | Credential::ApiKey { .. } => {
                    eprintln!("  -> {} (credential: {})", request.method, cred);
                }
            }
        }

        let response = match parse_request(&line) {
            Ok(request) => {
                let mut core = core.lock().expect("core mutex poisoned");
                RpcDispatcher::dispatch(&mut *core, &request)
            }
            Err(error) => {
                let response = nexum_protocol::RpcResponse::error(
                    None,
                    nexum_protocol::RpcErrorObject::parse_error(error.to_string()),
                );
                Some(response)
            }
        };

        if let Some(response) = response {
            let encoded = serialize_response(&response)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            stream.write_all(encoded.as_bytes())?;
            stream.write_all(b"\n")?;
            stream.flush()?;
        }
    }

    Ok(())
}

fn parse_cli_flags() -> (ServerConfig, PathBuf, bool, bool) {
    let mut config = ServerConfig::default();
    let mut config_path = PathBuf::new();
    let mut show_version = false;
    let mut show_help = false;
    let mut has_cli_port = false;
    let mut has_cli_data_dir = false;
    let mut has_cli_max_connections = false;
    let mut has_cli_require_auth = false;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--config" => {
                i += 1;
                if i < args.len() {
                    config_path = PathBuf::from(&args[i]);
                }
            }
            "--port" => {
                has_cli_port = true;
                i += 1;
                if i < args.len()
                    && let Ok(port) = args[i].parse::<u16>()
                {
                    config.port = port;
                }
            }
            "--data-dir" => {
                has_cli_data_dir = true;
                i += 1;
                if i < args.len() {
                    config.data_dir = PathBuf::from(&args[i]);
                }
            }
            "--max-connections" => {
                has_cli_max_connections = true;
                i += 1;
                if i < args.len()
                    && let Ok(max) = args[i].parse::<usize>()
                {
                    config.max_connections = max;
                }
            }
            "--require-auth" => {
                has_cli_require_auth = true;
                i += 1;
                if i < args.len()
                    && let Ok(req) = args[i].parse::<bool>()
                {
                    config.require_auth = req;
                }
            }
            "--version" => show_version = true,
            "--help" | "-h" => show_help = true,
            _ => {}
        }
        i += 1;
    }

    // Load from config file if provided, applying file defaults only where CLI didn't override
    if !config_path.is_empty()
        && let Ok(file_config) = ServerConfig::from_file(config_path.to_str().unwrap_or(""))
    {
        if !has_cli_port {
            config.port = file_config.port;
        }
        if !has_cli_data_dir {
            config.data_dir = file_config.data_dir;
        }
        if !has_cli_max_connections {
            config.max_connections = file_config.max_connections;
        }
        if !has_cli_require_auth {
            config.require_auth = file_config.require_auth;
        }
    }

    (config, config_path, show_version, show_help)
}

fn print_usage() {
    eprintln!("usage: nexum-server [OPTIONS]");
    eprintln!("Options:");
    eprintln!("  --config PATH       Load config from file");
    eprintln!("  --port PORT         Server port (default: 39100)");
    eprintln!("  --data-dir PATH     Data directory (default: ./data)");
    eprintln!("  --max-connections N Max concurrent connections (default: 100)");
    eprintln!("  --require-auth BOOL Require authentication for all requests");
    eprintln!("  --version           Show server and protocol version");
    eprintln!("  --help, -h          Show this help");
}

fn main() -> std::io::Result<()> {
    let (config, _config_path, show_version, show_help) = parse_cli_flags();

    if show_version {
        eprintln!("nexum-server {}", env!("CARGO_PKG_VERSION"));
        eprintln!("protocol {}", RpcDispatcher::version());
        return Ok(());
    }

    if show_help {
        print_usage();
        return Ok(());
    }

    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir).ok();

    let address = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(&address)?;
    let core = Arc::new(Mutex::new(
        Core::new(nexum_core::nexum_scheduler::SchedulerConfig::default())
            .map_err(|error| std::io::Error::other(error.to_string()))?,
    ));

    eprintln!("Nexum server listening on {address}");
    eprintln!(
        "max_connections: {}, data_dir: {:?}",
        config.max_connections, config.data_dir
    );

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let core = Arc::clone(&core);
                std::thread::spawn(move || {
                    if let Err(error) = handle_connection(stream, core) {
                        eprintln!("connection error: {error}");
                    }
                });
            }
            Err(error) => eprintln!("accept error: {error}"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn config_from_default_returns_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 39100);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.data_dir, PathBuf::from("./data"));
    }

    #[test]
    fn server_version_returns_correct_value() {
        assert_eq!(RpcDispatcher::version(), "1");
    }

    #[test]
    fn parse_config_file_with_all_fields() {
        let dir = std::env::temp_dir().join("nexum-server-test-config");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("config.txt");
        {
            let mut f = std::fs::File::create(&config_file).unwrap();
            writeln!(f, "port=8080").unwrap();
            writeln!(f, "max_connections=50").unwrap();
            writeln!(f, "data_dir=/tmp/nexum").unwrap();
        }
        let config = ServerConfig::from_file(config_file.to_str().unwrap()).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.data_dir, PathBuf::from("/tmp/nexum"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_config_file_uses_defaults_for_missing_fields() {
        let dir = std::env::temp_dir().join("nexum-server-test-partial");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("config.txt");
        {
            let mut f = std::fs::File::create(&config_file).unwrap();
            writeln!(f, "port=9090").unwrap();
            // Only port is set, max_connections and data_dir should use defaults
        }
        let config = ServerConfig::from_file(config_file.to_str().unwrap()).unwrap();
        assert_eq!(config.port, 9090);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.data_dir, PathBuf::from("./data"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_config_file_ignores_comments() {
        let dir = std::env::temp_dir().join("nexum-server-test-comments");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("config.txt");
        {
            let mut f = std::fs::File::create(&config_file).unwrap();
            writeln!(f, "# This is a comment").unwrap();
            writeln!(f, "port=4444").unwrap();
        }
        let config = ServerConfig::from_file(config_file.to_str().unwrap()).unwrap();
        assert_eq!(config.port, 4444);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_config_file_returns_error_for_missing_file() {
        let result = ServerConfig::from_file("/nonexistent/path/config.txt");
        assert!(result.is_err());
    }

    #[test]
    fn parse_args_returns_default_config_without_flags() {
        let (config, config_path, show_version, show_help) = parse_cli_flags();
        assert_eq!(config.port, 39100);
        assert_eq!(config.max_connections, 100);
        assert!(config_path.is_empty());
        assert!(!show_version);
        assert!(!show_help);
    }
}
