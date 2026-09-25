use nexum_core::nexum_domain::TaskId;
use nexum_core::nexum_storage::{SqliteRepository, TaskRepository};
use nexum_core::nexum_task::TaskState;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    root: PathBuf,
    data: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nexum-server-test-{}-{stamp}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        let data = path.join("data");
        fs::create_dir(&path).unwrap();
        fs::create_dir(&data).unwrap();
        fs::create_dir(path.join("downloads")).unwrap();
        Self { root: path, data }
    }

    fn path(&self) -> &Path {
        &self.data
    }

    fn output_path(&self, name: &str) -> PathBuf {
        self.root.join("downloads").join(name)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct ServerProcess {
    child: Child,
    port: u16,
}

impl ServerProcess {
    fn start(data_dir: &Path) -> Self {
        let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = reservation.local_addr().unwrap().port();
        drop(reservation);

        let child = Command::new(env!("CARGO_BIN_EXE_nexum-server"))
            .arg("--port")
            .arg(port.to_string())
            .arg("--data-dir")
            .arg(data_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut server = Self { child, port };
        for _ in 0..100 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return server;
            }
            if let Some(status) = server.child.try_wait().unwrap() {
                panic!("server exited before listening: {status}");
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("server did not listen on port {port}");
    }

    fn request(&self, method: &str, params: Option<Value>) -> Value {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let request = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
        writeln!(stream, "{request}").unwrap();
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        response
    }

    fn call(&self, method: &str, params: Option<Value>) -> Value {
        let response = self.request(method, params);
        assert!(response.get("error").is_none(), "{response}");
        response["result"].clone()
    }

    fn task_state(&self, id: &str) -> String {
        self.call("task.get", Some(json!({"id": id})))["state"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn wait_for_state(&self, id: &str, expected: &str) {
        for _ in 0..100 {
            if self.task_state(id) == expected {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("task {id} did not reach {expected}");
    }

    fn stop(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }
}

struct HttpFixture {
    source: String,
    connected: Receiver<()>,
    release: Sender<()>,
    worker: JoinHandle<()>,
}

impl HttpFixture {
    fn new(body: &'static [u8]) -> Self {
        Self::with_status("200 OK", body)
    }

    fn with_status(status: &'static str, body: &'static [u8]) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let source = format!("http://{}/file", listener.local_addr().unwrap());
        let (connected_tx, connected) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut method = [0u8; 3];
            stream.read_exact(&mut method).unwrap();
            assert_eq!(&method, b"GET");
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.flush().unwrap();
            connected_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let _ = stream.write_all(body);
        });
        Self {
            source,
            connected,
            release,
            worker,
        }
    }

    fn wait_until_connected(&self) {
        self.connected.recv_timeout(Duration::from_secs(3)).unwrap();
    }

    fn finish(self) {
        self.release.send(()).unwrap();
        self.worker.join().unwrap();
    }
}

struct ChunkedHttpFixture {
    source: String,
    first_chunk_sent: Receiver<()>,
    release: Sender<()>,
    worker: JoinHandle<()>,
}

impl ChunkedHttpFixture {
    fn new(first: Vec<u8>, second: Vec<u8>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let source = format!("http://{}/file", listener.local_addr().unwrap());
        let (first_chunk_tx, first_chunk_sent) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut method = [0u8; 3];
            stream.read_exact(&mut method).unwrap();
            assert_eq!(&method, b"GET");
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                first.len() + second.len()
            )
            .unwrap();
            stream.write_all(&first).unwrap();
            stream.flush().unwrap();
            first_chunk_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let _ = stream.write_all(&second);
        });
        Self {
            source,
            first_chunk_sent,
            release,
            worker,
        }
    }

    fn wait_until_first_chunk(&self) {
        self.first_chunk_sent
            .recv_timeout(Duration::from_secs(3))
            .unwrap();
    }

    fn finish(self) {
        self.release.send(()).unwrap();
        self.worker.join().unwrap();
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn tasks_survive_server_restart_and_active_states_requeue() {
    let dir = TestDir::new();
    let mut server = ServerProcess::start(dir.path());
    for id in ["created", "queued", "downloading", "paused"] {
        server.call(
            "task.create",
            Some(json!({
                "id": id,
                "source": format!("https://example.com/{id}"),
                "destination": dir.output_path(id).to_string_lossy(),
            })),
        );
    }
    for id in ["downloading", "paused", "queued"] {
        server.call("task.queue", Some(json!({"id": id})));
    }
    server.stop();

    let mut repository = SqliteRepository::open(dir.path().join("nexum.sqlite")).unwrap();
    for (id, state) in [
        ("downloading", TaskState::Downloading),
        ("paused", TaskState::Paused),
    ] {
        let mut task = repository.get(&TaskId::from(id)).unwrap().unwrap();
        task.state = state;
        repository.update(task).unwrap();
    }
    drop(repository);

    let mut restarted = ServerProcess::start(dir.path());
    assert_eq!(restarted.task_state("created"), "Created");
    for id in ["queued", "downloading", "paused"] {
        assert_eq!(restarted.task_state(id), "Queued");
    }
    restarted.stop();

    let repository = SqliteRepository::open(dir.path().join("nexum.sqlite")).unwrap();
    for id in ["queued", "downloading", "paused"] {
        let stored = repository.get(&TaskId::from(id)).unwrap().unwrap();
        assert_eq!(stored.state, TaskState::Queued);
    }
    drop(repository);
}

#[test]
fn http_download_completes_without_blocking_other_rpc_and_releases_slots() {
    let dir = TestDir::new();
    let mut server = ServerProcess::start(dir.path());
    let body = b"Nexum HTTP download";

    for index in 0..4 {
        let fixture = HttpFixture::new(body);
        let id = format!("http-{index}");
        let destination = dir.output_path(&id);
        server.call(
            "task.create",
            Some(json!({
                "id": id,
                "source": fixture.source,
                "destination": destination.to_string_lossy(),
            })),
        );
        server.call("task.queue", Some(json!({"id": id})));
        assert_eq!(server.call("task.start", None), id);
        fixture.wait_until_connected();
        assert_eq!(server.call("server.version", None), "1");
        assert_eq!(server.task_state(&id), "Downloading");
        assert!(
            server
                .request("task.pause", Some(json!({"id": id})))
                .get("error")
                .is_some()
        );
        assert!(
            server
                .request("task.remove", Some(json!({"id": id})))
                .get("error")
                .is_some()
        );
        fixture.finish();
        server.wait_for_state(&id, "Completed");
        assert_eq!(fs::read(&destination).unwrap(), body);
        let result = server.call("task.get", Some(json!({"id": id})));
        assert_eq!(result["downloaded_bytes"], body.len());
    }

    server.stop();
    let mut restarted = ServerProcess::start(dir.path());
    for index in 0..4 {
        assert_eq!(restarted.task_state(&format!("http-{index}")), "Completed");
    }
    restarted.stop();
}

#[test]
fn http_download_persists_incremental_progress_for_rpc_reads() {
    let dir = TestDir::new();
    let mut server = ServerProcess::start(dir.path());
    let first = vec![b'a'; 32 * 1024];
    let second = vec![b'b'; 32 * 1024];
    let fixture = ChunkedHttpFixture::new(first, second);
    let id = "progress-http";
    let destination = dir.output_path(id);
    server.call(
        "task.create",
        Some(json!({
            "id": id,
            "source": fixture.source,
            "destination": destination.to_string_lossy(),
        })),
    );
    server.call("task.queue", Some(json!({"id": id})));
    assert_eq!(server.call("task.start", None), id);
    fixture.wait_until_first_chunk();

    let mut observed = 0;
    for _ in 0..100 {
        let task = server.call("task.get", Some(json!({"id": id})));
        observed = task["downloaded_bytes"].as_u64().unwrap();
        if observed > 0 {
            assert_eq!(task["state"], "Downloading");
            assert!(observed < 64 * 1024);
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(observed > 0, "no incremental progress was persisted");

    fixture.finish();
    server.wait_for_state(id, "Completed");
    assert_eq!(fs::read(&destination).unwrap().len(), 64 * 1024);
    server.stop();
}

#[test]
fn unsupported_sources_stay_queued_and_http_errors_requeue() {
    let dir = TestDir::new();
    let mut server = ServerProcess::start(dir.path());
    server.call(
        "task.create",
        Some(json!({
            "id": "magnet",
            "source": "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567",
            "destination": dir.output_path("magnet").to_string_lossy(),
        })),
    );
    server.call("task.queue", Some(json!({"id": "magnet"})));
    assert!(server.request("task.start", None).get("error").is_some());
    assert_eq!(server.task_state("magnet"), "Queued");

    let fixture = HttpFixture::with_status("503 Service Unavailable", b"retry later");
    let destination = dir.output_path("failed-http");
    server.call(
        "task.create",
        Some(json!({
            "id": "failed-http",
            "source": fixture.source,
            "destination": destination.to_string_lossy(),
        })),
    );
    server.call("task.queue", Some(json!({"id": "failed-http"})));
    assert_eq!(server.call("task.start", None), "failed-http");
    assert_eq!(server.task_state("magnet"), "Queued");
    fixture.wait_until_connected();
    fixture.finish();
    server.wait_for_state("failed-http", "Queued");
    assert!(!destination.exists());
    let failed = server.call("task.get", Some(json!({"id": "failed-http"})));
    assert!(
        failed["error"]
            .as_str()
            .is_some_and(|message| message.contains("HTTP GET returned 503"))
    );
    server.call("task.remove", Some(json!({"id": "magnet"})));
    server.stop();

    let mut restarted = ServerProcess::start(dir.path());
    let failed = restarted.call("task.get", Some(json!({"id": "failed-http"})));
    assert_eq!(failed["state"], "Queued");
    assert!(
        failed["error"]
            .as_str()
            .is_some_and(|message| message.contains("HTTP GET returned 503"))
    );
    restarted.stop();
}

#[test]
fn http_download_cannot_replace_server_database_or_lock() {
    let dir = TestDir::new();
    let mut server = ServerProcess::start(dir.path());
    for file in ["nexum.sqlite", "nexum.lock"] {
        let id = format!("protected-{file}");
        server.call(
            "task.create",
            Some(json!({
                "id": id,
                "source": "http://127.0.0.1:9/file",
                "destination": dir.path().join(file).to_string_lossy(),
            })),
        );
        server.call("task.queue", Some(json!({"id": id})));
        assert!(server.request("task.start", None).get("error").is_some());
        assert_eq!(server.task_state(&id), "Queued");
        server.call("task.remove", Some(json!({"id": id})));
    }
    assert_eq!(server.call("server.version", None), "1");
    server.stop();

    let mut restarted = ServerProcess::start(dir.path());
    assert_eq!(restarted.call("server.version", None), "1");
    restarted.stop();
}

#[test]
fn http_downloads_to_the_same_destination_do_not_overlap() {
    let dir = TestDir::new();
    let mut server = ServerProcess::start(dir.path());
    let first = HttpFixture::new(b"first response");
    let second = HttpFixture::new(b"second response");
    let destination = dir.output_path("shared");
    for (id, source) in [("first", &first.source), ("second", &second.source)] {
        server.call(
            "task.create",
            Some(json!({
                "id": id,
                "source": source,
                "destination": destination.to_string_lossy(),
            })),
        );
        server.call("task.queue", Some(json!({"id": id})));
    }

    assert_eq!(server.call("task.start", None), "first");
    first.wait_until_connected();
    assert!(server.request("task.start", None).get("error").is_some());
    assert_eq!(server.task_state("second"), "Queued");
    first.finish();
    server.wait_for_state("first", "Completed");
    assert_eq!(fs::read(&destination).unwrap(), b"first response");

    assert_eq!(server.call("task.start", None), "second");
    second.wait_until_connected();
    second.finish();
    server.wait_for_state("second", "Completed");
    assert_eq!(fs::read(&destination).unwrap(), b"second response");
    server.stop();
}

fn server_output_with_timeout(data_dir: &Path, port: Option<u16>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nexum-server"));
    command.arg("--data-dir").arg(data_dir);
    if let Some(port) = port {
        command.arg("--port").arg(port.to_string());
    }
    let mut child = command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    for _ in 0..100 {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        thread::sleep(Duration::from_millis(50));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    panic!("server did not exit within five seconds");
}

#[test]
fn second_server_cannot_recover_a_live_servers_tasks() {
    let dir = TestDir::new();
    let fixture = HttpFixture::new(b"slow download");
    let mut first = ServerProcess::start(dir.path());
    first.call(
        "task.create",
        Some(json!({
            "id": "running",
            "source": fixture.source,
            "destination": dir.output_path("running").to_string_lossy(),
        })),
    );
    first.call("task.queue", Some(json!({"id": "running"})));
    first.call("task.start", None);
    fixture.wait_until_connected();
    assert_eq!(first.task_state("running"), "Downloading");

    let output = server_output_with_timeout(dir.path(), Some(0));
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("cannot acquire data directory lock"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(first.task_state("running"), "Downloading");
    let repository = SqliteRepository::open(dir.path().join("nexum.sqlite")).unwrap();
    assert_eq!(
        repository
            .get(&TaskId::from("running"))
            .unwrap()
            .unwrap()
            .state,
        TaskState::Downloading
    );
    drop(repository);

    first.stop();
    fixture.finish();
    let mut restarted = ServerProcess::start(dir.path());
    assert_eq!(restarted.task_state("running"), "Queued");
    restarted.stop();
}

#[test]
fn startup_fails_when_data_dir_is_a_file() {
    let dir = TestDir::new();
    let invalid_dir = dir.path().join("not-a-directory");
    fs::write(&invalid_dir, "occupied").unwrap();
    let output = server_output_with_timeout(&invalid_dir, None);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot create data directory"));
}

#[test]
fn startup_fails_when_database_cannot_be_opened() {
    let dir = TestDir::new();
    fs::write(dir.path().join("nexum.sqlite"), "not a SQLite database").unwrap();
    let output = server_output_with_timeout(dir.path(), None);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("nexum.sqlite"));
}
