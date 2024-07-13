use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("Server running on port 4221");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

// handle the client request
fn handle_client(mut stream: TcpStream) {
    let request = read_request(&mut stream);
    println!("request: {}", request);
    let response = handle_response(&request);
    send_response(&mut stream, &response);
}

// read the HTTP request from the buffer
fn read_request(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    String::from_utf8_lossy(&buffer).to_string()
}

// handle the HTTP request and generate the response
fn handle_response(request: &str) -> Vec<u8> {
    let request_path = request
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<&str>>()[1];
    println!("request path: {}", request_path);

    let request_accept_encoding = request
        .lines()
        .find(|line| line.starts_with("Accept-Encoding:") || line.starts_with("accept-encoding:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|encodings| encodings.split(',').map(str::trim).collect::<Vec<&str>>())
        .unwrap_or_default()
        .iter()
        .find(|&&encoding| encoding == "gzip")
        .unwrap_or(&"")
        .to_string();
    println!("request accept encoding: {}", request_accept_encoding);
    match request_path {
        "/" => "HTTP/1.1 200 OK\r\n\r\n".to_string().into_bytes(),
        "/user-agent" => {
            let user_agent = request
                .lines()
                .find(|line| line.starts_with("User-Agent") || line.starts_with("user-agent"))
                .unwrap()
                .split_whitespace()
                .collect::<Vec<&str>>()[1..]
                .join(" ");
            println!("user agent: {}", user_agent);
            generate_text_response(&user_agent, &request_accept_encoding)
        }
        _ if request_path.starts_with("/echo/") => {
            let echoed_str = &request_path[6..];
            println!("echoed string: {}", echoed_str);
            generate_text_response(echoed_str, &request_accept_encoding)
        }
        _ if request_path.starts_with("/files/") => {
            let filename = &request_path[6..];
            println!("filename: {}", filename);
            let env_args: Vec<String> = env::args().collect();
            let mut dir = env_args[2].clone();
            dir.push_str(&filename);
            println!("dir: {}", dir);
            let request_method = request
                .lines()
                .next()
                .unwrap()
                .split_whitespace()
                .collect::<Vec<&str>>()[0];
            println!("request method: {}", request_method);
            match request_method {
                "GET" => {
                    let file = match fs::read(dir) {
                        Ok(content) => content,
                        Err(_) => return generate_not_found_response(),
                    };
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        file.len(),
                        String::from_utf8_lossy(&file).to_string()
                    ).into_bytes()
                }
                "POST" => {
                    let request_body = request.split("\r\n\r\n").collect::<Vec<&str>>()[1];
                    let content_length = request
                        .lines()
                        .find(|line| {
                            line.starts_with("Content-Length") || line.starts_with("content-length")
                        })
                        .unwrap()
                        .split_whitespace()
                        .collect::<Vec<&str>>()[1]
                        .parse::<usize>()
                        .unwrap();
                    let contents = request_body
                        .chars()
                        .take(content_length)
                        .collect::<String>();
                    fs::write(dir, contents).unwrap();
                    "HTTP/1.1 201 Created\r\n\r\n".to_string().into_bytes()
                }
                _ => generate_not_found_response(),
            }
        }
        _ => generate_not_found_response(),
    }
}

// generate the text response
fn generate_text_response(response_body: &str, request_accept_encoding: &str) -> Vec<u8> {
    match request_accept_encoding {
        "gzip" => {
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(response_body.as_bytes()).unwrap();
            let compressed_bytes = encoder.finish().unwrap();
            let mut response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n",
                compressed_bytes.len()
            )
            .into_bytes();
            response.extend_from_slice(&compressed_bytes);
            response
        }
        _ => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            response_body
        )
        .into_bytes(),
    }
}

fn generate_not_found_response() -> Vec<u8> {
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string().into_bytes()
}

// send the response to the client
fn send_response(stream: &mut TcpStream, response: &[u8]) {
    stream.write_all(response).unwrap();
}
