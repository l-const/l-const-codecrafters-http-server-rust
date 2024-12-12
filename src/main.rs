use core::str;
#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_client(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

const HTTP_OK: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";
const HTTP_NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\n\r\n";

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 150];
    let result = stream.read(&mut buf);
    if let Ok(n) = result {
        let body = str::from_utf8(&buf[..n]).unwrap();
        println!("Read {n} bytes!!");
        println!("Buf size {} ", buf.len());
        match extract_request_path(body) {
            HttpResponseCode::HttpNotFound => {
                let _ = stream.write(HTTP_NOT_FOUND);
            }
            HttpResponseCode::HttpOk => {
                let _ = stream.write(HTTP_OK);
            }
        }
    }

    let _ = stream.shutdown(std::net::Shutdown::Both);
}

const CRLF: &str = "\r\n";

fn extract_request_path(request_body: &str) -> HttpResponseCode {
    let request_line = get_request_line(request_body);
    let request_target = get_request_target(request_line);
    dbg!(request_line);
    dbg!(request_target);
    if matches!(request_target, "/") {
        HttpResponseCode::HttpOk
    } else {
        HttpResponseCode::HttpNotFound
    }
}

fn get_request_line(request_body: &str) -> &str {
    request_body.split_terminator(CRLF).take(1).next().unwrap()
}

fn get_request_target(request_line: &str) -> &str {
    request_line.split_terminator(" ").take(2).last().unwrap()
}

fn find_echo_path(input: &str) -> Option<&str> {
    return None;
}

#[derive(Debug, PartialEq, Eq)]
enum HttpResponseCode {
    HttpOk = 200,
    HttpNotFound = 404,
}

#[cfg(test)]
mod tests {
    use crate::{extract_request_path, get_request_line, get_request_target, HttpResponseCode};

    static HTTP_OK_BODY: &str =
        "GET / HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n";
    static HTTP_OK_NOT_FOUND: &str = "GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n";
    static HTTP_OK_TARGET_LINE: &str = "GET / HTTP/1.1";
    static HTTP_OK_TARGET_PATH: &str = "/";

    #[test]
    fn test_extract_correct_target_line() {
        assert_eq!(get_request_line(HTTP_OK_BODY), HTTP_OK_TARGET_LINE);
    }

    #[test]
    fn test_extract_correct_target_path() {
        assert_eq!(get_request_target(HTTP_OK_TARGET_LINE), HTTP_OK_TARGET_PATH);
    }

    #[test]
    fn test_extract_request_body_http_not_found() {
        assert_eq!(
            extract_request_path(HTTP_OK_NOT_FOUND),
            HttpResponseCode::HttpNotFound
        )
    }

    #[test]
    fn test_extract_request_body_http_ok() {
        assert_eq!(extract_request_path(HTTP_OK_BODY), HttpResponseCode::HttpOk)
    }
}
