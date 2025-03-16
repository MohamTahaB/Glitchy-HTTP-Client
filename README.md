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

Given that each chunk is at most 64KB, and data does not exceed 1MB, data in its integrity should be constructed in at most 16 chunks. I have chosen a timeout of 30s, which makes downloading the data feasible for connections as slow as ~ 300Kbits/s.

Here is an example of logs output by the client when downloading the chunks (a more verbose logging can be obtained by uncommenting body checksum size throughout chunk downloading in the client code ...)

```
Checksum: 941ab02e167f745edde6abe043502da83b3e25f68a2424b3e20d259fd45b3b24
chunk no 1 downloaded successfully !!!
chunk no 2 downloaded successfully !!!
chunk no 3 downloaded successfully !!!
chunk no 4 downloaded successfully !!!
chunk no 5 downloaded successfully !!!
chunk no 6 downloaded successfully !!!
chunk no 7 downloaded successfully !!!
chunk no 8 downloaded successfully !!!
chunk no 9 downloaded successfully !!!
chunk no 10 downloaded successfully !!!
chunk no 11 downloaded successfully !!!
chunk no 12 downloaded successfully !!!
chunk no 13 downloaded successfully !!!
Data downloaded successfully !!! length: 847329

```