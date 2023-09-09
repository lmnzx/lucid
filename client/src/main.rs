use std::convert::TryInto;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::vec::Vec;

const K_MAX_MSG: usize = 4096;

fn msg(msg: &str) {
    eprintln!("{}", msg);
}

fn die(msg: &str) -> ! {
    let err = io::Error::last_os_error();
    eprintln!("[{}] {}", err.raw_os_error().unwrap_or(0), msg);
    std::process::abort();
}

fn read_full(fd: &mut TcpStream, mut buf: &mut [u8]) -> io::Result<()> {
    let mut n = buf.len();
    while n > 0 {
        match fd.read(buf) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "unexpected EOF",
                ))
            }
            Ok(rv) => {
                n -= rv;
                buf = &mut buf[rv..];
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn write_all(fd: &mut TcpStream, buf: &[u8]) -> io::Result<()> {
    let mut n = buf.len();
    let mut buf = buf;
    while n > 0 {
        match fd.write(buf) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::WriteZero, "write zero bytes")),
            Ok(rv) => {
                n -= rv;
                buf = &buf[rv..];
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn send_req(fd: &mut TcpStream, cmd: &[String]) -> io::Result<()> {
    let mut len = 4u32;
    for s in cmd.iter() {
        len += 4 + s.len() as u32;
    }
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

    write_all(fd, &wbuf)?;
    Ok(())
}

enum SerType {
    Nil,
    Err { code: i32, message: String },
    Str(String),
    Int(i64),
    Arr(Vec<u8>),
}

fn on_response(data: &[u8]) -> Result<SerType, &'static str> {
    if data.is_empty() {
        return Err("bad response");
    }
    match data[0] {
        0 => Ok(SerType::Nil),
        1 => {
            if data.len() < 1 + 8 {
                return Err("bad response");
            }

            let code = i32::from_le_bytes(data[1..5].try_into().unwrap());
            let len = u32::from_le_bytes(data[5..9].try_into().unwrap());
            if data.len() < 1 + 8 + len as usize {
                return Err("bad response");
            }
            let message = String::from_utf8_lossy(&data[9..(9 + len as usize)]).to_string();
            Ok(SerType::Err { code, message })
        }
        2 => {
            if data.len() < 1 + 4 {
                return Err("bad response");
            }
            let len = u32::from_le_bytes(data[1..5].try_into().unwrap());
            if data.len() < 1 + 4 + len as usize {
                return Err("bad response");
            }
            let value = String::from_utf8_lossy(&data[5..(5 + len as usize)]).to_string();
            Ok(SerType::Str(value))
        }
        3 => {
            if data.len() < 1 + 8 {
                return Err("bad response");
            }
            let val = i64::from_le_bytes(data[1..9].try_into().unwrap());
            Ok(SerType::Int(val))
        }
        4 => {
            if data.len() < 1 + 4 {
                return Err("bad response");
            }

            let len = u32::from_le_bytes(data[1..5].try_into().unwrap());
            let mut arr_bytes = 1 + 4;
            let mut arr = Vec::new();
            for _ in 0..len {
                let rv = on_response(&data[arr_bytes..]);
                match rv {
                    Ok(value) => {
                        arr_bytes += match value {
                            SerType::Nil => 1,
                            SerType::Err { code, message } => {
                                arr.extend_from_slice(&[0]);
                                arr.extend_from_slice(&code.to_le_bytes());
                                let message_bytes = message.as_bytes();
                                let message_len = message_bytes.len() as u32;
                                arr.extend_from_slice(&message_len.to_le_bytes());
                                arr.extend_from_slice(message_bytes);
                                1 + 8 + message_len as usize
                            }
                            SerType::Str(value) => {
                                arr.extend_from_slice(&[2]);
                                let value_bytes = value.as_bytes();
                                let value_len = value_bytes.len() as u32;
                                arr.extend_from_slice(&value_len.to_le_bytes());
                                arr.extend_from_slice(value_bytes);
                                1 + 4 + value_len as usize
                            }
                            SerType::Int(value) => {
                                arr.extend_from_slice(&[3]);
                                arr.extend_from_slice(&value.to_le_bytes());
                                1 + 8
                            }
                            SerType::Arr(value) => {
                                arr.extend_from_slice(&[4]);
                                arr.extend_from_slice(&value);
                                1 + 4 + value.len()
                            }
                        };
                    }
                    Err(_) => return Err("bad response"),
                }
            }
            Ok(SerType::Arr(arr))
        }
        _ => Err("bad response"),
    }
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
        SerType::Arr(_) => println!("(arr) len={}", len),
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
