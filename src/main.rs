use core::str;
use regex::Regex;
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

const SERVER_HOST: &str = "127.0.0.1";
const SERVER_PORT: &str = "8080";
const TIMEOUT: u64 = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch length and checksum from /info endpoint
    let (length, checksum) = fetch_info()?;

    println!("Length: {}", length);
    println!("Checksum: {}", checksum);

    // Fetch data from / endpoint
    let data = fetch_data(length)?;

    println!("Data length: {}", data.len());

    Ok(())
}

fn fetch_info() -> Result<(usize, String), Box<dyn std::error::Error>> {
    // Create a new connection for the /info request
    let mut stream = TcpStream::connect(format!("{}:{}", SERVER_HOST, SERVER_PORT))?;

    // Send the /info request
    let request = format!(
        "GET /info HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        SERVER_HOST
    );
    stream.write_all(request.as_bytes())?;

    // Read the entire response
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;

    // Convert the response to a string
    let response = str::from_utf8(&response)?;

    // Extract length and checksum using regex
    let rg = Regex::new(r#""length"\s*:\s*([0-9]+),\s*"sha256"\s*:\s*"([0-9a-zA-Z]+)"#).unwrap();
    match rg.captures(response) {
        None => Err("checksum/length regex does not match the request...".into()),
        Some(captures) => {
            let length = captures.get(1).unwrap().as_str().parse::<usize>()?;
            let checksum = captures.get(2).unwrap().as_str().to_string();
            Ok((length, checksum))
        }
    }
}

fn fetch_data(content_length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a new connection for the / request
    let mut stream = TcpStream::connect(format!("{}:{}", SERVER_HOST, SERVER_PORT))?;

    // Set a timeout to drop the stream if reading tokes too long ...
    let timeout_duration = Duration::from_secs(TIMEOUT);
    let _ = stream.set_read_timeout(Some(timeout_duration.clone()));

    // Send the / request
    let request = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        SERVER_HOST
    );
    stream.write_all(request.as_bytes())?;

    // Read the response headers
    let mut buffer = [0; 1024];
    let mut headers = Vec::new();
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        headers.extend_from_slice(&buffer[..bytes_read]);
        if headers.ends_with(b"\r\n\r\n") {
            break;
        }
    }

    // Read the response body
    let mut body = Vec::new();
    let mut bytes_read = 0;

    // Since it is very likely the data is buggy, a fixed timer is useful to avoid indefinite loops to fetch empty sets of bytes
    let start_time = Instant::now();
    while bytes_read < content_length {
        if start_time.elapsed() > timeout_duration {
            // At this point, it is very likely that we received all the bytes from the buggy server...
            return Ok(body);
        }
        match stream.read(&mut buffer) {
            Ok(bytes) => {
                body.extend_from_slice(&buffer[..bytes]);
                bytes_read += bytes;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Handle timeout
                return Err("Read operation timed out".into());
            }
            Err(e) => {
                // Handle other errors
                return Err(e.into());
            }
        }
    }

    Ok(body)
}
