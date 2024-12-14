use core::str;
use std::net::TcpListener;
use std::{
    io::{Read, Write},
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
const HTTP_BAD_REQUEST: &[u8] = b"HTTP/1.1 400 Bad Request\r\n\r\n";
const CONTENT_TYPE: &str = "Content-Type: text/plain\r\n";
const CONTENT_LENGTH: &str = "Content-Length: {n_bytes}\r\n";
const CRLF: &str = "\r\n";

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 150];
    let result = stream.read(&mut buf);
    if let Ok(n) = result {
        let body = str::from_utf8(&buf[..n]).unwrap();
        println!("Read {n} bytes!!");
        println!("Buf size {} ", buf.len());
        handle_body(&mut stream, body);
    }

    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn handle_body(stream: &mut TcpStream, body: &str) {
    match extract_request_path(body) {
        (_, HttpResponseCode::HttpNotFound) => {
            let _ = stream.write(HTTP_NOT_FOUND);
        }
        (path, HttpResponseCode::HttpOk) => {
            if matches!(path, "/") {
                let _ = stream.write(HTTP_OK);
            } else {
                let response_body = construct_multiline_response(path);
                dbg!(&response_body);
                let _ = stream.write(response_body.as_bytes());
            }
        }
        (_, HttpResponseCode::HttpBadRequest) => {
            let _ = stream.write(HTTP_BAD_REQUEST);
        }
    }
}

fn construct_multiline_response(response_body: &str) -> String {
    let n_bytes = response_body.bytes().len();
    let content_length = format!("Content-Length: {n_bytes}\r\n");
    let http_code_slice = str::from_utf8(HTTP_OK).unwrap();
    let len = http_code_slice.len() - 2;
    //http status code \r\n
    // headers(content type + length) + \r\n for each header
    // \r\n
    //body
    format!(
        "{}{}{}\r\n{}",
        &http_code_slice[0..len],
        CONTENT_TYPE,
        content_length,
        response_body
    )
}

fn extract_request_path(request_body: &str) -> (&str, HttpResponseCode) {
    let request_line = if let Ok(request) = get_request_line(request_body) {
        request
    } else {
        return ("", HttpResponseCode::HttpBadRequest);
    };
    let request_target = if let Ok(request) = get_request_target(request_body) {
        request
    } else {
        return ("", HttpResponseCode::HttpBadRequest);
    };
    dbg!(request_line);
    dbg!(request_target);
    if matches!(request_target, "/") {
        ("/", HttpResponseCode::HttpOk)
    } else {
        // check to see if it is an echo path
        let echo_path_opt = find_echo_path(request_target);
        if let Some(path) = echo_path_opt {
            return (path, HttpResponseCode::HttpOk);
        }
        ("", HttpResponseCode::HttpNotFound)
    }
}

fn get_request_line(request_body: &str) -> Result<&str, &str> {
    // BY SPECIFICATION OF THE HTTP protocol we epxect to have this
    //true
    request_body
        .split_terminator(CRLF)
        .take(1)
        .next()
        .ok_or("bad request")
}

fn get_request_target(request_line: &str) -> Result<&str, &str> {
    request_line
        .split_terminator(" ")
        .take(2)
        .last()
        .ok_or("bad request")
}

fn find_echo_path(target_path: &str) -> Option<&str> {
    target_path
        .split_once("/echo/")
        .map(|(_, remainder)| remainder)
}

#[derive(Debug, PartialEq, Eq)]
enum HttpResponseCode {
    HttpOk = 200,
    HttpBadRequest = 400,
    HttpNotFound = 404,
}

#[cfg(test)]
mod tests {
    use crate::{
        construct_multiline_response, extract_request_path, get_request_line, get_request_target,
        HttpResponseCode,
    };

    static HTTP_OK_BODY: &str =
        "GET / HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n";
    static HTTP_OK_NOT_FOUND: &str = "GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n";
    static HTTP_OK_TARGET_LINE: &str = "GET / HTTP/1.1";
    static HTTP_OK_TARGET_PATH: &str = "/";
    static ECHO_REQUEST: &str = "GET /echo/abc HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n";

    #[test]
    fn test_extract_correct_target_line() {
        assert_eq!(get_request_line(HTTP_OK_BODY).unwrap(), HTTP_OK_TARGET_LINE);
    }

    #[test]
    fn test_extract_correct_target_path() {
        assert_eq!(
            get_request_target(HTTP_OK_TARGET_LINE).unwrap(),
            HTTP_OK_TARGET_PATH
        );
    }

    #[test]
    fn test_extract_request_body_http_not_found() {
        assert_eq!(
            extract_request_path(HTTP_OK_NOT_FOUND).1,
            HttpResponseCode::HttpNotFound
        )
    }

    #[test]
    fn test_extract_request_body_http_ok() {
        assert_eq!(
            extract_request_path(HTTP_OK_BODY).1,
            HttpResponseCode::HttpOk
        )
    }

    #[test]
    fn test_extract_request_body_http_echo() {
        assert_eq!(extract_request_path(ECHO_REQUEST).0, "abc")
    }

    static expected_response: &str =
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc";

    #[test]
    fn test_construct_multiline_response_echo_abc() {
        assert_eq!(construct_multiline_response("abc"), expected_response);
    }
}
