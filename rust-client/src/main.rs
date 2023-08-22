use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

const K_MAX_MSG: usize = 4096;

fn send_req(stream: &mut TcpStream, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let len = text.len() as u32;
    if len > K_MAX_MSG as u32 {
        return Err("message too long".into());
    }

    stream.write_all(&(len as u32).to_le_bytes())?;
    stream.write_all(text.as_bytes())?;
    Ok(())
}

fn read_res(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut len_bytes = [0u8; 4]; // 4 bytes for u32
    stream.read_exact(&mut len_bytes)?;

    let len = u32::from_le_bytes(len_bytes);
    if len > K_MAX_MSG as u32 {
        return Err("message too long".into());
    }

    let mut response = vec![0u8; len as usize];
    stream.read_exact(&mut response)?;
    println!("server says: {}", String::from_utf8_lossy(&response));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 1234));
    let mut stream = TcpStream::connect(addr)?;

    let query_list = ["hello", "world", "foo", "bar"];
    for query in &query_list {
        send_req(&mut stream, query)?;
    }

    for _ in &query_list {
        read_res(&mut stream)?;
    }

    Ok(())
}
