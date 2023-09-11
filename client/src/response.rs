use crate::sertype::SerType;

pub fn on_response(data: &[u8]) -> Result<SerType, &'static str> {
    if data.is_empty() {
        return Err("bad response");
    }

    match data[0] {
        0 => Ok(SerType::Nil),
        1 => parse_err_response(data),
        2 => parse_str_response(data),
        3 => parse_int_response(data),
        // 4 => parse_float_response(data),
        5 => parse_arr_response(data),
        _ => Err("bad response"),
    }
}

fn parse_err_response(data: &[u8]) -> Result<SerType, &'static str> {
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

fn parse_str_response(data: &[u8]) -> Result<SerType, &'static str> {
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

fn parse_int_response(data: &[u8]) -> Result<SerType, &'static str> {
    if data.len() < 1 + 8 {
        return Err("bad response");
    }

    let val = i64::from_le_bytes(data[1..9].try_into().unwrap());
    Ok(SerType::Int(val))
}

// fn parse_float_response(data: &[u8]) -> Result<SerType, &'static str> {
//     if data.len() < 1 + 8 {
//         return Err("bad response");
//     }

//     let val = f64::from_le_bytes(data[1..9].try_into().unwrap());
//     Ok(SerType::Float(val))
// }

fn parse_arr_response(data: &[u8]) -> Result<SerType, &'static str> {
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
                let (bytes_added, elem_len) = match value {
                    SerType::Nil => (1, 0),
                    SerType::Err { code, message } => {
                        arr.extend_from_slice(&[0]);
                        arr.extend_from_slice(&code.to_le_bytes());
                        let message_bytes = message.as_bytes();
                        let message_len = message_bytes.len() as u32;
                        arr.extend_from_slice(&message_len.to_le_bytes());
                        arr.extend_from_slice(message_bytes);
                        (1 + 8 + message_len as usize, 1)
                    }
                    SerType::Str(value) => {
                        arr.extend_from_slice(&[2]);
                        let value_bytes = value.as_bytes();
                        let value_len = value_bytes.len() as u32;
                        arr.extend_from_slice(&value_len.to_le_bytes());
                        arr.extend_from_slice(value_bytes);
                        (1 + 4 + value_len as usize, 1)
                    }
                    SerType::Int(value) => {
                        arr.extend_from_slice(&[3]);
                        arr.extend_from_slice(&value.to_le_bytes());
                        (1 + 8, 1)
                    }
                    // SerType::Float(value) => {
                    //     arr.extend_from_slice(&[4]);
                    //     arr.extend_from_slice(&value.to_le_bytes());
                    //     (1 + 8, 1)
                    // }
                    SerType::Arr(value) => {
                        arr.extend_from_slice(&[4]);
                        arr.extend_from_slice(&value);
                        (1 + 4 + value.len(), value.len() / 6)
                    }
                };
                arr_bytes += bytes_added;
                arr_bytes += elem_len * 6; // Advance bytes for array elements
            }
            Err(e) => {
                println!("Error: {}", e);
                return Err("bad response");
            }
        }
    }

    Ok(SerType::Arr(arr))
}
