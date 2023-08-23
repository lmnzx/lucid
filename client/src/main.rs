use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

const K_MAX_MSG: usize = 4096;

fn send_req(stream: &mut TcpStream, cmd: &[String]) -> Result<(), Box<dyn Error>> {
    let mut len = 4u32;
    for s in cmd {
        len += 4 + s.len() as u32;
    }

    if len > K_MAX_MSG as u32 {
        return Err("message too long".into());
    }

    let mut wbuf = Vec::with_capacity(4 + len as usize);
    wbuf.extend(&len.to_le_bytes());
    wbuf.extend(&(cmd.len() as u32).to_le_bytes());

    for s in cmd {
        let p = s.len() as u32;
        wbuf.extend(&p.to_le_bytes());
        wbuf.extend(s.as_bytes());
    }

    stream.write_all(&wbuf)?;

    Ok(())
}

fn read_res(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut rbuf = [0u8; 4 + K_MAX_MSG + 1];
    stream.read_exact(&mut rbuf[..4])?;

    let len = u32::from_le_bytes([rbuf[0], rbuf[1], rbuf[2], rbuf[3]]);
    if len > K_MAX_MSG as u32 {
        return Err("Response too long".into());
    }

    stream.read_exact(&mut rbuf[4..(4 + len) as usize])?;

    let rescode = u32::from_le_bytes([rbuf[4], rbuf[5], rbuf[6], rbuf[7]]);
    let response = String::from_utf8_lossy(&rbuf[8..(4 + len) as usize]);

    println!("server says: [{}] {}", rescode, response);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 1234));
    let mut stream = TcpStream::connect(addr)?;

    let cmd: Vec<String> = std::env::args().skip(1).collect();

    send_req(&mut stream, &cmd)?;
    read_res(&mut stream)?;

    Ok(())
}
