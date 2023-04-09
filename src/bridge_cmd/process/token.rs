use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

/// 类型转换错误
#[derive(Debug)]
pub enum InvalidFormat {
    InvalidLength,
    InvalidChar(char),
}

pub struct Token(u32);
impl Token {
    pub const LENGTH: usize = 6;
    pub const CHARS: &[u8; 16] = b"0123456789abcdef";
    pub fn new() -> Self {
        let t = chrono::Local::now().timestamp_subsec_nanos();
        Token(t.to_le() >> 8)
    }
    pub fn val(&self) -> u32 {
        self.0
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::with_capacity(Self::LENGTH);
        let src = self.0.to_le_bytes();
        for b in &src[..Self::LENGTH / 2] {
            buf.push(Self::CHARS[(*b >> 4) as usize] as char);
            buf.push(Self::CHARS[(*b & 0xf) as usize] as char);
        }
        f.write_str(&buf)
    }
}
impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({} -> {})", self.0, self)
    }
}
impl FromStr for Token {
    type Err = InvalidFormat;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.len() != Self::LENGTH {
            return Err(InvalidFormat::InvalidLength);
        }

        fn assert(c: char) -> Result<u8, InvalidFormat> {
            match Token::CHARS.binary_search(&(c as u8)) {
                Ok(x) => Ok(x as u8),
                _ => Err(InvalidFormat::InvalidChar(c)),
            }
        }
        let c: Vec<char> = s.chars().collect();
        let mut buf = [0; 4];
        for (x, n) in buf.iter_mut().take(Self::LENGTH / 2).enumerate() {
            let cx = x * 2;
            *n = assert(c[cx])? << 4 | assert(c[cx + 1])?;
        }
        Ok(Self(u32::from_le_bytes(buf)))
    }
}

#[test]
fn ts_parse() {
    for _ in 0..10 {
        let t = Token::new();
        let c = match Token::from_str(&t.to_string()) {
            Ok(c) => c,
            Err(e) => panic!("{e:?}"),
        };
        assert_eq!(t.val(), c.val());
        std::thread::sleep(std::time::Duration::from_millis(64));
    }
}
