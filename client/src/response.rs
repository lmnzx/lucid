use std::error::Error;
use std::str;

pub fn on_response(data: &[u8]) -> Result<usize, Box<dyn Error>> {
    if data.is_empty() {
        println!("bad response");
        return Err("Bad response".into());
    }
    match data[0] {
        0 => {
            println!("(nil)");
            Ok(1)
        }
        1 => {
            if data.len() < 9 {
                println!("bad response");
                return Err("Bad response".into());
            }
            let mut code_bytes = [0u8; 4];
            code_bytes.copy_from_slice(&data[1..5]);
            let code = i32::from_le_bytes(code_bytes);
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&data[5..9]);
            let len = u32::from_le_bytes(len_bytes);
            if data.len() < (9 + len as usize) {
                println!("bad response");
                return Err("Bad response".into());
            }
            let err_msg = str::from_utf8(&data[9..(9 + len as usize)])?;
            println!("(err) {} {}", code, err_msg);
            Ok(9 + len as usize)
        }
        2 => {
            if data.len() < 5 {
                println!("bad response");
                return Err("Bad response".into());
            }
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&data[1..5]);
            let len = u32::from_le_bytes(len_bytes);
            if data.len() < (5 + len as usize) {
                println!("bad response");
                return Err("Bad response".into());
            }
            let str_data = str::from_utf8(&data[5..(5 + len as usize)])?;
            println!("(str) {}", str_data);
            Ok(5 + len as usize)
        }
        3 => {
            if data.len() < 9 {
                println!("bad response");
                return Err("Bad response".into());
            }
            let mut val_bytes = [0u8; 8];
            val_bytes.copy_from_slice(&data[1..9]);
            let val = i64::from_le_bytes(val_bytes);
            println!("(int) {}", val);
            Ok(9)
        }
        4 => {
            if data.len() < 9 {
                println!("bad response");
                return Err("Bad response".into());
            }
            let mut val_bytes = [0u8; 8];
            val_bytes.copy_from_slice(&data[1..9]);
            let val = f64::from_le_bytes(val_bytes);
            println!("(dbl) {}", val);
            Ok(9)
        }
        5 => {
            if data.len() < 5 {
                println!("bad response");
                return Err("Bad response".into());
            }
            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&data[1..5]);
            let len = u32::from_le_bytes(len_bytes);
            println!("(arr) len={}", len);
            let mut arr_bytes = 5;
            for _ in 0..len {
                let rv = on_response(&data[arr_bytes..])?;
                arr_bytes += rv;
            }
            println!("(arr) end");
            Ok(arr_bytes)
        }
        _ => {
            println!("bad response");
            Err("Bad response".into())
        }
    }
}
