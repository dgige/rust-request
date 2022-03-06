extern crate url;

use std::io::{self, Result, ErrorKind};

pub enum Protocol {
    Http,
    Https
}

pub struct Url {
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub path: String
}

impl Url {
    pub fn new(url: &str) -> Result<Url> {
        let parsed_url = match url::Url::parse(url) {
            Ok(url) => url,
            Err(e) => {
                let err = io::Error::new(ErrorKind::InvalidInput, e);
                return Err(err);
            }
        };

        let protocol = match &*parsed_url.scheme() {
            "http" => Protocol::Http,
            "https" => Protocol::Https,
            _ => {
                let err = io::Error::new(ErrorKind::InvalidInput, "The protocol is not supported.");
                return Err(err);
            }
        };
        
        let host = match parsed_url.domain() {
            Some(domain) => domain,
            None => {
                let err = io::Error::new(ErrorKind::InvalidInput, "The URL is invalid.");
                return Err(err);
            }
        };

        let port = match parsed_url.port() {
            Some(port) => port,
            None => {
                match protocol {
                    Protocol::Http => 80_u16,
                    Protocol::Https => 443_u16
                }
            }
        };

        let mut path = String::new();
        path.push_str(parsed_url.path());
        if let Some(q) = parsed_url.query() {
            path.push('?');
            path.push_str(q);
        }

        Ok(Url {
            protocol,
            host: host.to_string(),
            port,
            path,
        })
    }
}
