use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

pub fn start() {
  // bind() in this scenario works like the new() in that it will return a new TcpListener instance
  let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

  for stream in listener.incoming() {
    println!("Connection established!");
    let stream: TcpStream = stream.unwrap();
    handle_connection(stream);
  }
}

fn handle_connection(mut stream: TcpStream) {
  let buf_reader = BufReader::new(&mut stream);

  // The browser signals the end of an HTTP request by sending two newline characters in a row,
  // so to get one request from the stream, we take lines until we get a line that is the empty string.
  let http_request: Vec<_> = buf_reader
    .lines()
    .map(|result| result.unwrap())
    .take_while(|line| !line.is_empty())
    .collect();

  println!("Request: {:#?}", http_request);

  let resp_status = "HTTP/1.1 200 OK";
  let resp_content = fs::read_to_string("hello.html").unwrap();
  let content_len = resp_content.len();

  let response = format!(
    "{}\r\nContent-Length: {}\r\n\r\n{}",
    resp_status, content_len, resp_content
  );
  stream.write(response.as_bytes()).unwrap();
}
