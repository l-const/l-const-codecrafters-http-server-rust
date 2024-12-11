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

fn handle_client(mut stream: TcpStream) {
    let mut buf = Vec::with_capacity(128);
    'retry: while let Ok(n) =  stream.read(&mut buf) {
        println!("Read {n} bytes!!");
        println!("Buf size {} ", buf.len());
        println!("Buffer contents: {:?}", &buf);
        stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
        if n == 0 {
           break 'retry;     
        } else {
            break;
        }
    }
    // if let Ok(n) = result {
    //     println!("Read {n} bytes!!");
    //     println!("Buffer contents: {:?}", &buf);
    // } else {
    //     eprintln!("Error reading input: {:?}", result.err().unwrap());
    // }
}
