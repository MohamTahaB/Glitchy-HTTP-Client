# Glitchy HTTP Server Client.
## Overview
The following repo is an HTTP Client written using Rust, that downloads some data from a [glitchy server](https://gist.github.com/vladimirlagunov/dcdf90bb19e9de306344d46f20920dce), and checks data integrity through validating the SHA256 hash, provided by the server itself from one hand, and after receiving the potentially missing incomplete data from the other hand.

## Precisions
One point to be taken into account, is that I took the liberty to add an endpoint at the server `/info`, that provides the data checksum.

```
def do_GET(self):
    if self.path == "/info":
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                response = {
                        "sha256" : hashlib.sha256(self.data).hexdigest(),
                        }
                self.wfile.write(json.dumps(response).encode())
                return 
    // rest of do_GET...
```

## Client Implementation: Quick look:

A quick look at the server confirms that:
- Data consists of random bytes between 512 KB up to 1 MB,
- The server accepts the header `Range`, as precised by the prompt.
- The **Glitchiness** stems from the fact that, when the lenght of the chunk of data being queried exceeds 64 KB, the data being sent is the first `N` bytes of the data, where `N` is a random number between `64 * 1024` and the data's length (both included).

It is possible to naively query all the data in one go, but at this point, it is very unlikely to have the checksum checkout. However, one way to overcome glitchiness is through constructing data by chunks (in this case, we can go up to 64 KB before having heuristic behavior).

Since the prompt encouraged not to use any external libs, I tried doing that to a minimum: instead of using high level HTTP client crates such as `reqwest`, I have established connection with the server using `std::net::TcpStream` and relied on string parsing and/or regexes instead of `serde` for deserializing.