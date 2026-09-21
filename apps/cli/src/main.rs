use nexum_protocol::{parse_request, serialize_response, Credential, RpcRequest};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;

/// Simple config file management for the CLI.
pub struct Config {
    dir: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let dir = if let Ok(home) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(home).join("nexum")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("nexum")
        } else {
            PathBuf::from("./nexum")
        };
        Self { dir }
    }

    pub fn default_server_address(&self) -> Option<String> {
        self.read_config_value("default_server")
    }

    pub fn default_credential(&self) -> Option<Credential> {
        let scheme = self.read_config_value("default_auth_scheme")?;
        let token = self.read_config_value("default_auth_token")?;
        match scheme.as_str() {
            "Bearer" => Some(Credential::Bearer { token }),
            "ApiKey" => Some(Credential::ApiKey { key: token }),
            _ => None,
        }
    }

    pub fn set_server_address(&self, address: &str) -> Result<(), String> {
        self.write_config_value("default_server", address)
    }

    pub fn set_credential(&self, scheme: &str, token: &str) -> Result<(), String> {
        self.write_config_value("default_auth_scheme", scheme)?;
        self.write_config_value("default_auth_token", token)
    }

    fn read_config_value(&self, key: &str) -> Option<String> {
        let file = self.config_file();
        if !file.is_file() {
            return None;
        }
        std::fs::read_to_string(&file)
            .ok()
            .and_then(|content| {
                content.lines().find(|l| l.trim().starts_with(&format!("{key}=")))
            })
            .and_then(|line| line.trim().split_once('='))
            .map(|(_, value)| value.trim().to_owned())
    }

    fn write_config_value(&self, key: &str, value: &str) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|e| format!("cannot create config directory: {e}"))?;
        let file = self.config_file();
        if file.is_file() {
            let existing = std::fs::read_to_string(&file)
                .map_err(|e| format!("cannot read config: {e}"))?;
            let new_content = existing
                .lines()
                .map(|line| {
                    if line.trim().starts_with(&format!("{key}=")) {
                        format!("{key} = {value}")
                    } else {
                        line.to_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(&file, new_content)
                .map_err(|e| format!("cannot write config: {e}"))?;
        } else {
            std::fs::write(&file, format!("{key} = {value}"))
                .map_err(|e| format!("cannot write config: {e}"))?;
        }
        Ok(())
    }

    fn config_file(&self) -> PathBuf {
        self.dir.join("config.txt")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JsonRpcClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl JsonRpcClient {
    pub fn connect(address: &str) -> Result<Self, String> {
        let stream = TcpStream::connect(address)
            .map_err(|e| format!("cannot connect to Nexum server at {address}: {e}"))?;
        let reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
        Ok(Self { stream, reader })
    }

    pub fn call_with_credential(
        &mut self,
        id: u64,
        method: &str,
        params: Option<Value>,
        credential: Option<Credential>,
    ) -> Result<Value, String> {
        let request = RpcRequest::new(id, method, params).with_credential(credential);
        let payload = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        self.stream
            .write_all(payload.as_bytes())
            .map_err(|e| e.to_string())?;
        self.stream.write_all(b"\n").map_err(|e| e.to_string())?;
        self.stream.flush().map_err(|e| e.to_string())?;

        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        let response: nexum_protocol::RpcResponse =
            serde_json::from_str(&line).map_err(|e| e.to_string())?;
        if let Some(error) = response.error {
            return Err(format!("[{}] {}", error.code, error.message));
        }
        Ok(response.result.unwrap_or(Value::Null))
    }

    pub fn call(&mut self, id: u64, method: &str, params: Option<Value>) -> Result<Value, String> {
        self.call_with_credential(id, method, params, None)
    }
}

fn usage() {
    eprintln!("usage:");
    eprintln!("  nexum [--server ADDR] task list");
    eprintln!("  nexum [--server ADDR] task get ID");
    eprintln!("  nexum [--server ADDR] task create ID SOURCE DESTINATION");
    eprintln!("  nexum [--server ADDR] task queue ID");
    eprintln!("  nexum [--server ADDR] task start");
    eprintln!("  nexum [--server ADDR] task pause ID");
    eprintln!("  nexum [--server ADDR] task resume ID");
    eprintln!("  nexum [--server ADDR] task remove ID");
    eprintln!("  nexum [--server ADDR] config get-server");
    eprintln!("  nexum [--server ADDR] config set-server ADDR");
    eprintln!("  nexum [--server ADDR] auth set SCHEME TOKEN");
    eprintln!("  nexum [--server ADDR] auth clear");
    eprintln!("  nexum [--server ADDR] server ping");
    eprintln!("  nexum [--server ADDR] server version");
    eprintln!("  nexum [--server ADDR] server auth");
    eprintln!("  nexum --version");
    eprintln!("  nexum --help");
}

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.iter().any(|a| a == "--version") {
        eprintln!("nexum {}", env!("CARGO_PKG_VERSION"));
        eprintln!("protocol {}", nexum_protocol::RpcDispatcher::version());
        return;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        usage();
        return;
    }

    let mut address = String::from("127.0.0.1:39100");
    let mut credential: Option<Credential> = None;

    if args.len() >= 2 && args[0] == "--server" {
        address = args[1].clone();
        args.drain(0..2);
    } else {
        let config = Config::new();
        if let Some(default_addr) = config.default_server_address() {
            address = default_addr;
        }
        credential = config.default_credential();
    }

    if args.len() < 2
        || !matches!(
            args[0].as_str(),
            "task" | "config" | "auth" | "server"
        )
    {
        usage();
        std::process::exit(2);
    }

    let mut verify_version = true;
    let (method, params) = match (args[0].as_str(), args[1].as_str()) {
        ("task", "list") if args.len() == 2 => ("task.list", None),
        ("task", "get") if args.len() == 3 => {
            ("task.get", Some(serde_json::json!({ "id": args[2] })))
        }
        ("task", "create") if args.len() == 5 => (
            "task.create",
            Some(serde_json::json!({
                "id": args[2],
                "source": args[3],
                "destination": args[4]
            })),
        ),
        ("task", "queue") if args.len() == 3 => {
            ("task.queue", Some(serde_json::json!({ "id": args[2] })))
        }
        ("task", "start") if args.len() == 2 => ("task.start", None),
        ("task", "pause") if args.len() == 3 => {
            ("task.pause", Some(serde_json::json!({ "id": args[2] })))
        }
        ("task", "resume") if args.len() == 3 => {
            ("task.resume", Some(serde_json::json!({ "id": args[2] })))
        }
        ("task", "remove") if args.len() == 3 => {
            ("task.remove", Some(serde_json::json!({ "id": args[2] })))
        }
        ("server", "version") if args.len() == 2 => ("server.version", None),
        ("server", "auth") if args.len() == 2 => ("server.auth", None),
        ("server", "ping") if args.len() == 2 => {
            verify_version = false;
            ("server.version", None)
        }
        ("config", "get-server") if args.len() == 2 => {
            match get_server_address() {
                Ok(addr) => {
                    println!("{addr}");
                    return;
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        ("config", "set-server") if args.len() == 3 => {
            match Config::new().set_server_address(&args[2]) {
                Ok(()) => {
                    println!("Server address set to {}", args[2]);
                    return;
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        ("auth", "set") if args.len() == 4 => {
            match Config::new().set_credential(&args[2], &args[3]) {
                Ok(()) => {
                    println!("Authentication set: {}", args[2]);
                    return;
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        ("auth", "clear") if args.len() == 2 => {
            match Config::new().write_config_value("default_auth_scheme", "") {
                Ok(()) => {
                    println!("Authentication cleared");
                    return;
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            usage();
            std::process::exit(2);
        }
    };

    let mut client = match JsonRpcClient::connect(&address) {
        Ok(client) => client,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    if verify_version {
        if let Ok(response) = client.call_with_credential(0, "server.version", None, None) {
            if let Some(server_ver) = response.as_str() {
                let local_ver = nexum_protocol::RpcDispatcher::version();
                if server_ver != local_ver {
                    eprintln!(
                        "warning: server protocol v{server_ver} differs from client v{local_ver}"
                    );
                } else {
                    eprintln!("connected: server protocol v{server_ver}");
                }
            }
        }
    }

    match client.call_with_credential(1, method, params, credential) {
        Ok(value) => println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn get_server_address() -> Result<String, String> {
    Config::new()
        .default_server_address()
        .ok_or_else(|| "No server address configured. Use: nexum config set-server ADDR".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_without_file() {
        let config = Config::new();
        let _ = config.default_server_address();
    }

    #[test]
    fn rpc_client_is_constructible() {
        assert!(JsonRpcClient::connect("127.0.0.1:1").is_err());
    }
}
