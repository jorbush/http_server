use reqwest::blocking::Client;
use std::fs::File;
use std::io::Write;

const SERVER_ADDR: &str = "http://localhost:4221";

#[test]
fn test_respond_with_200_ok() {
    let client = Client::new();
    let response = client.get(SERVER_ADDR).send().unwrap();

    assert_eq!(response.status(), 200);
}

#[test]
fn test_respond_with_404_not_found() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/abcdefg", SERVER_ADDR))
        .send()
        .unwrap();
    assert_eq!(response.status(), 404);
}

#[test]
fn test_respond_with_body() {
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
}

#[test]
fn test_read_header() {
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
}

#[test]
fn test_concurrent_connections() {
    let handles: Vec<_> = (0..3)
        .map(|_| {
            std::thread::spawn(|| {
                let client = Client::new();
                let response = client.get(SERVER_ADDR).send().unwrap();
                assert_eq!(response.status(), 200);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_return_file_found() {
    let mut file = File::create("/tmp/foo").unwrap();
    file.write_all(b"Hello, World!").unwrap();

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
    assert_eq!(response.headers().get("Content-Length").unwrap(), "13");
    assert_eq!(response.text().unwrap(), "Hello, World!");
}

#[test]
fn test_return_file_not_found() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/files/non_existant_file", SERVER_ADDR))
        .send()
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[test]
fn test_read_request_body() {
    let client = Client::new();
    let response = client
        .post(&format!("{}/files/file_123", SERVER_ADDR))
        .body("12345")
        .header("Content-Type", "application/octet-stream")
        .send()
        .unwrap();

    assert_eq!(response.status(), 201);
}

#[test]
fn test_compression_headers_valid() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .header("Accept-Encoding", "gzip")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("Content-Encoding").unwrap(), "gzip");
}

#[test]
fn test_compression_headers_invalid() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .header("Accept-Encoding", "invalid-encoding")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("Content-Encoding").is_none());
}

#[test]
fn test_multiple_compression_schemes_one_valid() {
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
}

#[test]
fn test_multiple_compression_schemes_all_invalid() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/echo/abc", SERVER_ADDR))
        .header("Accept-Encoding", "invalid-encoding-1, invalid-encoding-2")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert!(response.headers().get("Content-Encoding").is_none());
}

#[test]
fn test_gzip_compression() {
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
}
