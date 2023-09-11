use std::convert::TryInto;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::vec::Vec;

mod response;
mod sertype;

use response::on_response;
use sertype::SerType;

const K_MAX_MSG: usize = 4096;

fn send_req(fd: &mut TcpStream, cmd: &[String]) -> io::Result<()> {
    let len: u32 = 4 + cmd.iter().map(|s| 4 + s.len() as u32).sum::<u32>();

    if len > K_MAX_MSG as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "message too long",
        ));
    }

    let mut wbuf = Vec::with_capacity(4 + len as usize);
    wbuf.extend_from_slice(&len.to_le_bytes());
    let n = cmd.len() as u32;
    wbuf.extend_from_slice(&n.to_le_bytes());

    for s in cmd.iter() {
        let p = s.len() as u32;
        wbuf.extend_from_slice(&p.to_le_bytes());
        wbuf.extend_from_slice(s.as_bytes());
    }

    fd.write_all(&wbuf)?;
    Ok(())
}

fn read_res(fd: &mut TcpStream) -> io::Result<()> {
    let mut rbuf = vec![0u8; 4 + K_MAX_MSG + 1];
    rbuf[4 + K_MAX_MSG] = 0; // Null-terminate the buffer for string conversion
    fd.read_exact(&mut rbuf[..4])?;
    let len = u32::from_le_bytes(rbuf[..4].try_into().unwrap());
    if len > K_MAX_MSG as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "message too long",
        ));
    }

    fd.read_exact(&mut rbuf[4..(4 + len as usize)])?;

    let rv = on_response(&rbuf[4..(4 + len as usize)]).unwrap();
    match rv {
        SerType::Nil => println!("(nil)"),
        SerType::Err { code, message } => println!("(err) {} {}", code, message),
        SerType::Str(value) => println!("(str) {}", value),
        SerType::Int(value) => println!("(int) {}", value),
        SerType::Float(value) => println!("(float) {}", value),
        SerType::Arr(value) => {
            println!("(arr) len={}", value.len() / 6);
            for chunk in value.chunks_exact(6) {
                let rv = on_response(chunk).unwrap();
                match rv {
                    SerType::Nil => println!("(nil)"),
                    SerType::Err { code, message } => println!("(err) {} {}", code, message),
                    SerType::Str(value) => println!("(str) {}", value),
                    SerType::Int(value) => println!("(int) {}", value),
                    SerType::Float(value) => println!("(float) {}", value),
                    SerType::Arr(_) => println!("(arr)"),
                }
            }
            println!("(arr) end")
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1234);
    let mut fd = TcpStream::connect(addr)?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    send_req(&mut fd, &args)?;
    read_res(&mut fd)?;

    Ok(())
}
