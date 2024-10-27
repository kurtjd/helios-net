use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Config {
    pub max_connections: usize,
    pub max_header_len: usize,
    pub max_body_len: usize,
    pub max_timeout: usize,
    pub ip: String,
    pub port_http: u16,
    pub port_https: u16,
    pub https_enabled: bool,
    pub server_root: String,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, ()> {
        let file = File::open(path).map_err(|_| ())?;

        let mut config = Self::default();
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            if let Some((name, value)) = line.split_once('=') {
                match name {
                    "max_connections" => config.max_connections = value.parse().map_err(|_| ())?,
                    "max_header_len" => config.max_header_len = value.parse().map_err(|_| ())?,
                    "max_body_len" => config.max_body_len = value.parse().map_err(|_| ())?,
                    "max_timeout" => config.max_timeout = value.parse().map_err(|_| ())?,
                    "ip" => config.ip = value.to_string(),
                    "port_http" => config.port_http = value.parse().map_err(|_| ())?,
                    "port_https" => config.port_https = value.parse().map_err(|_| ())?,
                    "https_enabled" => config.https_enabled = value.parse().map_err(|_| ())?,
                    "server_root" => config.server_root = value.to_string(),
                    _ => (),
                }
            }
        }

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_connections: 10,
            max_header_len: 8 * 1024,
            max_body_len: 1024 * 1024,
            max_timeout: 5,
            ip: String::from("127.0.0.1"),
            port_http: 1337,
            port_https: 31337,
            https_enabled: true,
            server_root: String::from("/var/www"),
        }
    }
}
