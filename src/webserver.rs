use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::{fs, thread};
use webserver::ThreadPool;

pub fn start() {
  // bind() in this scenario works like the new() in that it will return a new TcpListener instance
  let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
  let pool = ThreadPool::new(4); // creating a thread pool with a given number of threads

  // // ## Single-thread approach
  // for stream in listener.incoming() {
  //   println!("Connection established!");
  //   let stream: TcpStream = stream.unwrap();
  //   handle_connection(stream);
  // }

  // // ## Thread-spawning approach
  // for stream in listener.incoming() {
  //   println!("Connection established!");
  //   let stream: TcpStream = stream.unwrap();
  //   thread::spawn(|| handle_connection(stream));
  // }

  // // ## Thread pool approach
  // for stream in listener.incoming() {
  //   println!("> Connection established!");
  //   let stream: TcpStream = stream.unwrap();
  //   pool.execute(|| handle_connection(stream));
  // }

  // ## Gracefully shutting down example - after taking 2 requests
  for stream in listener.incoming().take(2) {
    let stream = stream.unwrap();
    pool.execute(|| handle_connection(stream));
  }

  println!("Shutting down the web server.")
}

fn handle_connection(mut stream: TcpStream) {
  let buf_reader = BufReader::new(&mut stream);

  // // The browser signals the end of an HTTP request by sending two newline characters in a row,
  // // so to get one request from the stream, we take lines until we get a line that is the empty string.
  // let http_request: Vec<_> = buf_reader
  //   .lines()
  //   .map(|result| result.unwrap())
  //   .take_while(|line| !line.is_empty())
  //   .collect();
  //
  // println!("Request: {:#?}", http_request);

  let request_line: String = buf_reader.lines().next().unwrap().unwrap();
  println!("Request: {:#?}", request_line);

  // let (resp_status_line, content_file) = if request_line == "GET / HTTP/1.1" {
  //   ("HTTP/1.1 200 OK", read_content_file("hello.html"))
  // } else {
  //   ("HTTP/1.1 404 NOT FOUND", read_content_file("404.html"))
  // };

  let (resp_status_line, content) = match &request_line[..] {
    "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", read_content_file("hello.html")),
    "GET /sleep HTTP/1.1" => {
      thread::sleep(std::time::Duration::from_secs(8));
      ("HTTP/1.1 200 OK", read_content_file("hello.html"))
    }
    _ => ("HTTP/1.1 404 NOT FOUND", read_content_file("404.html")),
  };

  let response = build_response(resp_status_line, content);
  send_response(stream, response);
}

fn read_content_file(filename: &str) -> String {
  fs::read_to_string(filename).unwrap()
}

fn build_response(status_line: &str, content: String) -> String {
  let content_len = content.len();
  format!(
    "{}\r\nContent-Length: {}\r\n\r\n{}",
    status_line, content_len, content
  )
}

fn send_response(mut stream: TcpStream, response: String) {
  stream.write_all(response.as_bytes()).unwrap();
}
