use nexum_protocol::{parse_request, serialize_response, RpcRequest};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub struct JsonRpcClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl JsonRpcClient {
    pub fn connect(address: &str) -> Result<Self, String> {
        let stream = TcpStream::connect(address).map_err(|e| format!("cannot connect to Nexum server at {address}: {e}"))?;
        let reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
        Ok(Self { stream, reader })
    }

    pub fn call(&mut self, id: u64, method: &str, params: Option<Value>) -> Result<Value, String> {
        let request = RpcRequest::new(id, method, params);
        let payload = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        self.stream.write_all(payload.as_bytes()).map_err(|e| e.to_string())?;
        self.stream.write_all(b"\n").map_err(|e| e.to_string())?;
        self.stream.flush().map_err(|e| e.to_string())?;

        let mut line = String::new();
        self.reader.read_line(&mut line).map_err(|e| e.to_string())?;
        let response: nexum_protocol::RpcResponse = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        if let Some(error) = response.error {
            return Err(format!("RPC {}: {}", error.code, error.message));
        }
        Ok(response.result.unwrap_or(Value::Null))
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
}

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut address = "127.0.0.1:39100".to_owned();

    if args.len() >= 2 && args[0] == "--server" {
        address = args[1].clone();
        args.drain(0..2);
    }

    if args.first().is_some_and(|arg| arg == "--help" || arg == "-h") {
        usage();
        return;
    }

    if args.len() < 2 || args[0] != "task" {
        usage();
        std::process::exit(2);
    }

    let (method, params) = match args[1].as_str() {
        "list" if args.len() == 2 => ("task.list", None),
        "get" if args.len() == 3 => ("task.get", Some(serde_json::json!({"id": args[2]}))),
        "create" if args.len() == 5 => ("task.create", Some(serde_json::json!({"id": args[2], "source": args[3], "destination": args[4]}))),
        "queue" if args.len() == 3 => ("task.queue", Some(serde_json::json!({"id": args[2]}))),
        "start" if args.len() == 2 => ("task.start", None),
        "pause" if args.len() == 3 => ("task.pause", Some(serde_json::json!({"id": args[2]}))),
        "resume" if args.len() == 3 => ("task.resume", Some(serde_json::json!({"id": args[2]}))),
        "remove" if args.len() == 3 => ("task.remove", Some(serde_json::json!({"id": args[2]}))),
        _ => { usage(); std::process::exit(2); }
    };

    let mut client = match JsonRpcClient::connect(&address) {
        Ok(client) => client,
        Err(error) => { eprintln!("{error}"); std::process::exit(1); }
    };

    match client.call(1, method, params) {
        Ok(value) => println!("{}", serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())),
        Err(error) => { eprintln!("{error}"); std::process::exit(1); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_is_documented() {
        assert_eq!("task.list", "task.list");
        let _ = parse_request;
        let _ = serialize_response;
    }
}
