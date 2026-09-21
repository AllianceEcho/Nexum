use nexum_core::Core;
use nexum_protocol::{parse_request, serialize_response, RpcDispatcher};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

fn handle_connection(mut stream: TcpStream, core: Arc<Mutex<Core>>) -> std::io::Result<()> {
    let reader_stream = stream.try_clone()?;
    let reader = BufReader::new(reader_stream);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
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

fn main() -> std::io::Result<()> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:39100".to_owned());
    let listener = TcpListener::bind(&address)?;
    let core = Arc::new(Mutex::new(Core::default()));

    eprintln!("Nexum server listening on {address}");

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
