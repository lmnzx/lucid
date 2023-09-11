pub enum SerType {
    Nil,
    Err { code: i32, message: String },
    Str(String),
    Int(i64),
    Float(f64),
    Arr(Vec<u8>),
}
