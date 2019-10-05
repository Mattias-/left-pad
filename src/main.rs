use std::io::{Write,Read};
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

type Headers = HashMap<String, String>;

#[derive(Debug)]
struct Request<'a> {
    headers: Headers,
    method: String,
    uri: String,
    version: String,
    body: &'a str,
}

#[derive(Debug)]
struct Response<'a> {
    headers: Headers,
    version: String,
    status: String,
    reason: String,
    body: &'a str,
}

enum HTTPStatus {
    OK = 200,
}

impl HTTPStatus {
    fn reason(&self) -> &str {
        match *self {
            HTTPStatus::OK => "OK",
        }
    }
}

impl<'a> Response<'a> {
    fn new() -> Self {
        Response {
            headers: Headers::new(),
            version: "HTTP/1.1".to_string(),
            status: (HTTPStatus::OK as i32).to_string(),
            reason: HTTPStatus::OK.reason().to_string(),
            body: "",
        }
    }

    fn status(mut self, status: HTTPStatus) -> Self {
        self.reason = status.reason().to_string();
        self.status = (status as i32).to_string();
        self
    }

    fn body(mut self, body: &'a str) -> Self {
        self.body = body;
        self
    }

    fn build(self) -> String {
        let res = self.version + " " + &self.status + " " + &self.reason + "\r\n\r\n" + self.body;
        return res;
    }
}

fn parse_headers(hs2: &str) -> Headers {
    hs2.split("\r\n")
        .map(|i| {
            let mut h = i.splitn(2, ": ").map(str::to_string);
            (
                h.nth(0).expect("Missing header key"),
                h.nth(0).expect("Missing header value"),
            )
        })
        .collect()
}

fn parse_body(body: &str) -> &str {
    return body.trim_end_matches('\0');
}

fn parse_request(request: &str) -> Request {
    let mut v = request.splitn(2, "\r\n\r\n");
    let prebod = v.nth(0).expect("No prebod");
    let bod = v.nth(0).expect("No body");

    let mut hs = prebod.splitn(2, "\r\n");
    let mut rrs = hs.nth(0).expect("No request line").split(" ");
    let hhs = hs.nth(0).expect("No header lines");

    let method = rrs.nth(0).expect("").to_string();
    let uri = rrs.nth(0).expect("").to_string();
    let version = rrs.nth(0).expect("").to_string();

    return Request {
        headers: parse_headers(hhs),
        method: method,
        uri: uri,
        version: version,
        body: parse_body(bod),
    };
}

fn left_pad(s: String, n: usize, padding: &str) -> String {
    return format!("{}{}", padding.repeat(n), s);
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    let r = String::from_utf8_lossy(&buffer[..]);
    let request = parse_request(&r);
    println!("{:?}", request);

    let x: Vec<&str> = request.uri.splitn(3, "/").collect();
    let i: usize = x[1].parse().unwrap();

    let res = Response::new()
        .status(HTTPStatus::OK)
        .body(&left_pad(String::from(request.body), i, x[2]))
        .build();

    stream.write(res.as_bytes()).unwrap();
    stream.flush().unwrap();
}
