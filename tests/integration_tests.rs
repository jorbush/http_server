use assert_cmd::Command;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use tempfile::tempdir;

const SERVER_ADDR: &str = "http://localhost:4221";

fn start_server(directory: String) {
    std::thread::spawn(move || {
        Command::cargo_bin("your_server")
            .unwrap()
            .arg("--directory")
            .arg(&directory)
            .unwrap();
    });

    // Wait for the server to start
    std::thread::sleep(Duration::from_secs(1));
}

fn stop_server() {
    // Connect and drop to trigger server shutdown
    let _ = TcpStream::connect("127.0.0.1:4221");
}

#[test]
fn test_respond_with_200_ok() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client.get(SERVER_ADDR).send().unwrap();

    assert_eq!(response.status(), 200);

    stop_server();
}

#[test]
fn test_respond_with_404_not_found() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/abcdefg", SERVER_ADDR))
        .send()
        .unwrap();

    assert_eq!(response.status(), 404);

    stop_server();
}

#[test]
fn test_respond_with_body() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "text/plain"
    );
    assert_eq!(response.headers().get("Content-Length").unwrap(), "3");
    assert_eq!(response.text().unwrap(), "abc");

    stop_server();
}

#[test]
fn test_read_header() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/user-agent", SERVER_ADDR))
        .header("User-Agent", "foobar/1.2.3")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "text/plain"
    );
    assert_eq!(response.headers().get("Content-Length").unwrap(), "12");
    assert_eq!(response.text().unwrap(), "foobar/1.2.3");

    stop_server();
}

#[test]
fn test_concurrent_connections() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let handles: Vec<_> = (0..3)
        .map(|_| {
            std::thread::spawn(|| {
                let client = Client::new();
                let response = client.get(SERVER_ADDR).send().unwrap();
                assert_eq!(response.status(), 200);
                assert_eq!(response.text().unwrap(), "HTTP/1.1 200 OK\r\n\r\n");
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    stop_server();
}

#[test]
fn test_return_file_found() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("foo");
    let mut file = File::create(file_path).unwrap();
    file.write_all(b"Hello, World!").unwrap();

    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/files/foo", SERVER_ADDR))
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/octet-stream"
    );
    assert_eq!(response.headers().get("Content-Length").unwrap(), "14");
    assert_eq!(response.text().unwrap(), "Hello, World!");

    stop_server();
}

#[test]
fn test_return_file_not_found() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/files/non_existant_file", SERVER_ADDR))
        .send()
        .unwrap();

    assert_eq!(response.status(), 404);

    stop_server();
}

#[test]
fn test_read_request_body() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("file_123");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"12345").unwrap();

    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .post(&format!("{}/files/file_123", SERVER_ADDR))
        .body("12345")
        .header("Content-Type", "application/octet-stream")
        .send()
        .unwrap();

    assert_eq!(response.status(), 201);

    stop_server();
}

#[test]
fn test_compression_headers_valid() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .header("Accept-Encoding", "gzip")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("Content-Encoding").unwrap(), "gzip");

    stop_server();
}

#[test]
fn test_compression_headers_invalid() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .header("Accept-Encoding", "invalid-encoding")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("Content-Encoding").is_none());

    stop_server();
}

#[test]
fn test_multiple_compression_schemes_one_valid() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .header(
            "Accept-Encoding",
            "invalid-encoding-1, gzip, invalid-encoding-2",
        )
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("Content-Encoding").unwrap(), "gzip");

    stop_server();
}

#[test]
fn test_multiple_compression_schemes_all_invalid() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .header("Accept-Encoding", "invalid-encoding-1, invalid-encoding-2")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("Content-Encoding").is_none());

    stop_server();
}

#[test]
fn test_gzip_compression() {
    let dir = tempdir().unwrap();
    start_server(dir.path().to_str().unwrap().to_owned());

    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .header("Accept-Encoding", "gzip")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("Content-Encoding").unwrap(), "gzip");

    let compressed_body = response.bytes().unwrap();
    // Verify the gzip header (first two bytes should be 0x1f and 0x8b)
    assert_eq!(&compressed_body[0..2], &[0x1f, 0x8b]);

    stop_server();
}
