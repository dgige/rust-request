//! Request
#![doc(html_root_url="https://ghmlee.github.io/rust-request/doc")]
extern crate openssl;

mod url;
pub mod response;

use openssl::ssl::SslConnector;
use std::io::{self, Write, Read, Result, ErrorKind};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::net::TcpStream;
use url::{Protocol, Url};
use response::Response;
use openssl::ssl::SslMethod;

pub fn post(url: &str,
            headers: &mut HashMap<String, String>,
            body: &[u8]) -> Result<Response> {
    connect("POST", &Url::new(url)?, headers, body)
}

pub fn get(url: &str,
           headers: &mut HashMap<String, String>) -> Result<Response> {
    connect("GET", &Url::new(url)?, headers, "".as_bytes())
}

pub fn put(url: &str,
            headers: &mut HashMap<String, String>,
            body: &[u8]) -> Result<Response> {
    connect("PUT", &Url::new(url)?, headers, body)
}

pub fn delete(url: &str,
            headers: &mut HashMap<String, String>) -> Result<Response> {
    connect("DELETE", &Url::new(url)?, headers, "".as_bytes())
}

pub fn options(url: &str,
           headers: &mut HashMap<String, String>) -> Result<Response> {
    connect("OPTIONS", &Url::new(url)?, headers, "".as_bytes())
}

pub fn head(url: &str,
           headers: &mut HashMap<String, String>) -> Result<Response> {
    connect("HEAD", &Url::new(url)?, headers, "".as_bytes())
}

pub fn list(url: &str,
    headers: &mut HashMap<String, String>) -> Result<Response> {
    connect("LIST", &Url::new(url)?, headers, "".as_bytes())
}

fn connect(method: &str,
           url: &Url,
           headers: &mut HashMap<String, String>,
           body: &[u8]) -> Result<Response> {
    // address
    let addr = format!("{}:{}", url.host, url.port);

    // host
    let host = match url.protocol {
        Protocol::Http => {
            match url.port {
                80 => url.host.clone(),
                _ => format!("{}:{}", url.host, url.port)
            }
        }
        Protocol::Https => {
            match url.port {
                443 => url.host.clone(),
                _ => format!("{}:{}", url.host, url.port)
            }
        }
    };
    
    // headers
    match headers.entry("Content-Length".to_string()) {
        Occupied(entry) => { entry.remove(); }
        Vacant(_) => { }
    }
    headers.insert("Content-Length".to_string(), format!("{}", body.len()));
    let mut http_headers = String::new();
    for header in headers.iter() {
        let key = header.0;
        let value = header.1;
        http_headers.push_str(&format!("\r\n{}: {}", key, value));
    }
    http_headers.push_str("\r\n\r\n");
    
    let http_content = format!("{} {} HTTP/1.1\r\nHost: {}{}",
                               method,
                               url.path,
                               host,
                               http_headers);
    
    let mut buf: Vec<u8> = Vec::new();
    for x in http_content.as_bytes() { buf.push(*x); }
    for x in body { buf.push(*x); }

    // stream
    let mut stream = match TcpStream::connect(&*addr) {
        Ok(stream) => stream,
        Err(e) => {
            let err = io::Error::new(ErrorKind::NotConnected, e);
            return Err(err);
        }
    };

    // raw
    let raw = match url.protocol {
        Protocol::Http => {
            let _ = stream.write(&*buf);
            read(&mut stream)?
        }
        Protocol::Https => {
            let context = match SslConnector::builder(SslMethod::tls()) {
                    Ok(context) => context.build(),
                    Err(e) => {
                        let err = io::Error::new(ErrorKind::NotConnected, e);
                        return Err(err);
                    }
                };
                let mut ssl_stream = match context.connect(&host, stream) {
                    Ok(stream) => stream,
                    Err(e) => {
                        let err = io::Error::new(ErrorKind::NotConnected, e);
                        return Err(err);
                    }
                };

            let _ = ssl_stream.write(&*buf);
            read(&mut ssl_stream)?
        }
    };

    // response
    let response = get_response(&raw)?;
    
    // redirect
    if response.status_code / 100 == 3 {
        let location = match response.headers.get("Location") {
            Some(location) => location,
            None => {
                let err = io::Error::new(ErrorKind::NotConnected,
                                         "Server returns an invalid response.");
                return Err(err);
            }
        };

        // it will support for a relative path
        return connect(method, &Url::new(location)?, headers, body);
    }
    
    Ok(response)
}

fn read<S: Read>(stream: &mut S) -> Result<String> {
    const BUFFER_SIZE: usize = 1024;
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut raw = String::new();
    loop {
        let len = match stream.read(&mut buffer) {
            Ok(size) => size,
            Err(e) => {
                let err = io::Error::new(ErrorKind::NotConnected, e);
                return Err(err);
            }
        };
        
        if len == 0 { break; }
        
        match std::str::from_utf8(&buffer[0 .. len]) {
            Ok(buf) => raw.push_str(buf),
            Err(e) => {
                let err = io::Error::new(ErrorKind::NotConnected, e);
                return Err(err);
            }
        }
    }

    Ok(raw)
}

fn get_response(raw: &str) -> Result<Response> {
    let http_response: Vec<&str> = raw.split("\r\n\r\n").collect();

    if http_response.len() < 2 {
        let err = io::Error::new(ErrorKind::InvalidInput,
                                 "Server returns an invalid response.");
        return Err(err);
    }
    
    let http_header = http_response[0];
    let http_body = http_response[1];
    let chunked_content_body: Vec<&str> = http_body.split("\r\n").collect();
    let mut content_body = String::new();

    if chunked_content_body.len() == 1 {
        content_body.push_str(http_body);
    } else {
        let mut index: i64 = 0;
        for chunk in chunked_content_body.iter() {
            index += 1;
            if index % 2 != 0 { continue; }
            content_body.push_str(chunk);
        }
    }

    let response = Response::new(http_header, &content_body);
    Ok(response)
}

#[test]
fn it_works() {
}
