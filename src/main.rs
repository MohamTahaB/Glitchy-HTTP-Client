use core::str;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

use hex;

const SERVER_HOST: &str = "127.0.0.1";
const SERVER_PORT: &str = "8080";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch length and checksum from /info endpoint
    let checksum = fetch_info()?;
    println!("Checksum: {}", checksum);

    // Fetch data from / endpoint
    let data = fetch_data()?;
    println!("Data length: {}", data.len());
    validate_data(data, checksum);

    Ok(())
}

fn fetch_info() -> Result<String, Box<dyn std::error::Error>> {
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

    // Extract checksum using regex
    let rg = Regex::new(r#""sha256"\s*:\s*"([0-9a-zA-Z]+)"#).unwrap();
    match rg.captures(response) {
        None => Err("checksum regex does not match the request...".into()),
        Some(captures) => {
            let checksum = captures.get(1).unwrap().as_str().to_string();
            Ok(checksum)
        }
    }
}

fn fetch_data() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a new connection for the / request
    let mut stream = TcpStream::connect(format!("{}:{}", SERVER_HOST, SERVER_PORT))?;

    // Send the / request
    let request = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        SERVER_HOST
    );
    stream.write_all(request.as_bytes())?;

    // Read the response body
    let mut body = Vec::new();

    let _ = stream.read_to_end(&mut body);

    // There is still a point to take into account: So far, the body contains headers + data, one workaround is to
    // parse into string and separate according to the special chars that separate body and headers

    let body_str = String::from_utf8_lossy(&body);

    let separator_pos = body_str
        .find("\r\n\r\n")
        .ok_or("Invalid response: no blank line found...")?;

    // then separate !
    let body = Vec::from(&body[separator_pos + 4..]);

    Ok(body)
}

fn validate_data(data: Vec<u8>, expected_checksum: String) {
    let mut hasher = Sha256::new();
    hasher.update(data);

    let message = if hex::encode(hasher.finalize()) == expected_checksum {
        "All good ! the message is OK!"
    } else {
        "UhOh :( something seems to be missing ..."
    };

    println!("{message}");
}
