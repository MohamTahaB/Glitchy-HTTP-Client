use core::str;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

use hex;

const SERVER_HOST: &str = "127.0.0.1";
const SERVER_PORT: &str = "8080";
const CHUNK: usize = 64 * 1024;
const TIMEOUT: u64 = 30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch length and checksum from /info endpoint
    let checksum = fetch_info()?;
    println!("Checksum: {}", checksum);

    // Fetch data from / endpoint
    let data = fetch_data(checksum)?;
    println!("Data downloaded successfully !!! length: {}", data.len());

    // Optional: at this point, it is possible to write the data to a file ...
    Ok(())
}

/// Queries the endpoint `/info` to get the data SHA256 hash.
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

/// Wrapper around chunk queries
/// Defines the container that will serve as a buffer for the downloaded chunks
/// and concatenates them.
/// A timeout over downloading all chunks has been implemented for safekeeping against infinite polling.
fn fetch_data(expected_checksum: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Define the starting point for queried chunks
    let mut starting: usize = 0;

    // Define the body
    let mut body = Vec::new();

    // For safekeeping, enforce a total timeout on downloading all the chunks, in case something goes wrong with the server connection.
    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(TIMEOUT);

    while !validate_data(&body, &expected_checksum) {
        // Check if the global timeout has been exceeded
        if start_time.elapsed() > timeout_duration {
            return Err("UhOooh !!! Global timeout exceeded while reading response body".into());
        }

        // Query a chunk
        let chunk = fetch_data_chunk(starting)?;
        // Optional: uncomment for verbose logging ...
        // println!("chunk of size: {}", chunk.len());
        body.extend(chunk.into_iter());
        // Optional: uncomment for verbose logging ...
        // println!("so far, body of size: {}", body.len());

        // On to the next chunk
        starting += CHUNK;
    }

    Ok(body)
}

fn fetch_data_chunk(starting: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a new connection for the / request
    let mut stream = TcpStream::connect(format!("{}:{}", SERVER_HOST, SERVER_PORT))?;

    // Send the / request
    let request = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nRange: bytes={}-{}\r\nConnection: close\r\n\r\n",
        SERVER_HOST,
        starting,
        starting + CHUNK
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

    // Print mssg
    println!(
        "chunk no {} downloaded successfully !!!",
        starting / CHUNK + 1
    );

    Ok(body)
}

fn validate_data(data: &Vec<u8>, expected_checksum: &String) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);

    // Optional: uncomment for verbose logging ...
    // println!(
    //     "body checksum so far = {}\nexpected = {}",
    //     hex::encode(hasher.clone().finalize()),
    //     expected_checksum
    // );

    hex::encode(hasher.finalize()) == *expected_checksum
}
