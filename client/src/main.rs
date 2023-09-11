use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::vec::Vec;

mod response;

use response::on_response;

const K_MAX_MSG: usize = 4096;

fn read_full(stream: &mut TcpStream, buf: &mut [u8]) -> Result<(), Box<dyn Error>> {
    let mut total_read = 0;
    while total_read < buf.len() {
        let read_result = stream.read(&mut buf[total_read..])?;
        if read_result == 0 {
            return Err("Unexpected EOF".into());
        }
        total_read += read_result;
    }
    Ok(())
}

fn write_all(stream: &mut TcpStream, buf: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut total_written = 0;
    while total_written < buf.len() {
        let write_result = stream.write(&buf[total_written..])?;
        if write_result == 0 {
            return Err("Write error".into());
        }
        total_written += write_result;
    }
    Ok(())
}

fn send_req(stream: &mut TcpStream, cmd: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut len = 4;
    for s in &cmd {
        len += 4 + s.len();
    }
    if len > K_MAX_MSG {
        return Err("Message too long".into());
    }

    let mut wbuf = vec![0; 4 + len];
    wbuf[..4].copy_from_slice(&(len as u32).to_le_bytes());
    let n = cmd.len() as u32;
    wbuf[4..8].copy_from_slice(&n.to_le_bytes());
    let mut cur = 8;
    for s in &cmd {
        let p = s.len() as u32;
        wbuf[cur..(cur + 4)].copy_from_slice(&p.to_le_bytes());
        wbuf[(cur + 4)..(cur + 4 + s.len())].copy_from_slice(s.as_bytes());
        cur += 4 + s.len();
    }
    write_all(stream, &wbuf)?;
    Ok(())
}

fn read_res(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    // 4 bytes header
    let mut rbuf = vec![0; 4 + K_MAX_MSG + 1];
    stream.read_exact(&mut rbuf[..4])?;
    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&rbuf[..4]);
    let len = u32::from_le_bytes(len_bytes);
    if len > K_MAX_MSG as u32 {
        println!("too long");
        return Err("Message too long".into());
    }

    // reply body
    read_full(stream, &mut rbuf[4..(4 + len as usize)])?;

    // print the result
    let rv = on_response(&rbuf[4..(4 + len as usize)])?;
    if rv > 0 && rv as u32 != len {
        println!("bad response");
        return Err("Bad response".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut stream = TcpStream::connect(SocketAddr::new("127.0.0.1".parse()?, 1234))?;
    send_req(&mut stream, args)?;
    read_res(&mut stream)?;
    Ok(())
}
