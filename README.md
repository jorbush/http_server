# HTTP Server

This is a simple HTTP server written in Rust.

## Requirements

- Rust

## Run the server

To run the server, you can use the following command:

```bash
cargo run -- --directory /tmp
```

In other terminal, you can use the following command to test the server:

```bash
curl -v http://localhost:4221
```

## Testing

```bash
cargo test
```

## Lint

```bash
cargo fmt
```

## Features

### Respond with 200 OK

```bash
curl -v http://localhost:4221
```

```
HTTP/1.1 200 OK\r\n\r\n
```

### Respond with 404 Not Found when the path is not defined

```bash
curl -v http://localhost:4221/abcdefg
```

```
HTTP/1.1 404 Not Found\r\n\r\n
```

### Respond with body

```bash
curl -v http://localhost:4221/echo/abc
```

```
HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc
```

### Read header

```bash
curl -v --header "User-Agent: foobar/1.2.3" http://localhost:4221/user-agent
```

```
HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 12\r\n\r\nfoobar/1.2.3
```

### Concurrent connections

```bash
$ (sleep 3 && printf "GET / HTTP/1.1\r\n\r\n") | nc localhost 4221 &
$ (sleep 3 && printf "GET / HTTP/1.1\r\n\r\n") | nc localhost 4221 &
$ (sleep 3 && printf "GET / HTTP/1.1\r\n\r\n") | nc localhost 4221 &
```

```
HTTP/1.1 200 OK\r\n\r\n
HTTP/1.1 200 OK\r\n\r\n
HTTP/1.1 200 OK\r\n\r\n
```

### Return a file

#### File found

```bash
echo -n 'Hello, World!' > /tmp/foo
curl -i http://localhost:4221/files/foo
```

```
HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 13\r\n\r\nHello, World!
```

#### File not found

```bash
curl -i http://localhost:4221/files/non_existant_file
```

```
HTTP/1.1 404 Not Found\r\n\r\n
```

### Read request body

```bash
curl -v --data "12345" -H "Content-Type: application/octet-stream" http://localhost:4221/files/file_123
```

```
HTTP/1.1 201 Created\r\n\r\n
```

### Compression headers

#### Valid Accept-Encoding header

```bash
curl -v -H "Accept-Encoding: gzip" http://localhost:4221/echo/abc
```

```
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Encoding: gzip
...
```

#### Invalid Accept-Encoding header

```bash
curl -v -H "Accept-Encoding: invalid-encoding" http://localhost:4221/echo/abc
```

```
HTTP/1.1 200 OK
Content-Type: text/plain

// Body omitted.
```

### Multiple compression schemes

#### Contains one valid encoding

```bash
curl -v -H "Accept-Encoding: invalid-encoding-1, gzip, invalid-encoding-2" http://localhost:4221/echo/abc
```

```
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Encoding: gzip

// Body omitted.
```

#### Contains all invalid encodings

```bash
curl -v -H "Accept-Encoding: invalid-encoding-1, invalid-encoding-2" http://localhost:4221/echo/abc
```

```
HTTP/1.1 200 OK
Content-Type: text/plain

// Body omitted.
```

### Gzip compression

```bash
curl -v -H "Accept-Encoding: gzip" http://localhost:4221/echo/abc | hexdump -C
```

```
HTTP/1.1 200 OK
Content-Encoding: gzip
Content-Type: text/plain
Content-Length: 23

1F 8B 08 00 00 00 00 00
00 03 4B 4C 4A 06 00 C2
41 24 35 03 00 00 00
```
