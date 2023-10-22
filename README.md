# LucidDB: A simple Key-Value Database

Lucid is a high-performance key-value database. It provides fast and reliable storage for your applications. The server is implemented in C++, ensuring efficiency and low latency, while the client component is written in Rust, offering a safe and ergonomic interface for interacting with the database.

### Getting Started
#### Prerequisites

- C++ compiler (supporting C++11 and above)
- Rust compiler (stable)
- CMake (for building the C++ server)
- Cargo (for building the Rust client)


#### Building the Server
```bash
git clone https://github.com/lmnzx/lucid.git && cd lucid
cmake .
make
```
To run the server (default port:1234)
```bash
./lucid
```

#### Building the Client
```bash
cd client 
cargo build
```

Using client
```
./target/debug/client get l
(nil)
./target/debug/client set l lmn
(nil)
./target/debug/client get l
(str) lmn
 ./target/debug/client keys
(arr) len=1
(str) l
(arr) end
./target/debug/client set m mln
(nil)
./target/debug/client keys
(arr) len=2
(str) m
(str) l
(arr) end
./target/debug/client del l
(int) 1
./target/debug/client keys
(arr) len=1
(str) m
(arr) end
```
