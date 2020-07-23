use std::io::{BufRead, Write};

#[derive(Debug)]
pub enum Error {
    InvalidCString,
    InvalidUTF8(std::string::FromUtf8Error),
    IOError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::InvalidUTF8(e)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait BinaryReader {
    fn read_u8(&mut self) -> Result<(usize, u8)>;
    fn read_u32(&mut self) -> Result<(usize, u32)>;
    fn read_u64(&mut self) -> Result<(usize, u64)>;
    fn read_c_string(&mut self) -> Result<(usize, String)>;
}

impl<T> BinaryReader for T
where
    T: BufRead,
{
    fn read_u8(&mut self) -> Result<(usize, u8)> {
        let mut buf: [u8; std::mem::size_of::<u8>()] = [0; std::mem::size_of::<u8>()];
        self.read_exact(&mut buf)?;
        Ok((buf.len(), buf[0]))
    }

    fn read_u32(&mut self) -> Result<(usize, u32)> {
        let mut buf: [u8; std::mem::size_of::<u32>()] = [0; std::mem::size_of::<u32>()];
        self.read_exact(&mut buf)?;
        Ok((buf.len(), u32::from_le_bytes(buf)))
    }

    fn read_u64(&mut self) -> Result<(usize, u64)> {
        let mut buf: [u8; std::mem::size_of::<u64>()] = [0; std::mem::size_of::<u64>()];
        self.read_exact(&mut buf)?;
        Ok((buf.len(), u64::from_le_bytes(buf)))
    }

    fn read_c_string(&mut self) -> Result<(usize, String)> {
        let mut buf = Vec::new();
        self.read_until(0, &mut buf)?;
        let bytes_read = buf.len();
        buf.pop(); //we dont need the null terminator
        Ok((bytes_read, String::from_utf8(buf)?))
    }
}

pub trait BinaryWriter {
    fn write_u8(&mut self, val: u8) -> Result<usize>;
    fn write_u32(&mut self, val: u32) -> Result<usize>;
    fn write_u64(&mut self, val: u64) -> Result<usize>;
    fn write_c_string(&mut self, val: &str) -> Result<usize>;
}

impl<T> BinaryWriter for T
where
    T: Write,
{
    fn write_u8(&mut self, val: u8) -> Result<usize> {
        let bytes = val.to_le_bytes();
        self.write_all(&bytes)?;
        Ok(bytes.len())
    }

    fn write_u32(&mut self, val: u32) -> Result<usize> {
        let bytes = val.to_le_bytes();
        self.write_all(&bytes)?;
        Ok(bytes.len())
    }

    fn write_u64(&mut self, val: u64) -> Result<usize> {
        let bytes = val.to_le_bytes();
        self.write_all(&bytes)?;
        Ok(bytes.len())
    }

    fn write_c_string(&mut self, val: &str) -> Result<usize> {
        let str_bytes = val.as_bytes();
        if str_bytes.contains(&0) {
            return Err(Error::InvalidCString);
        }

        self.write_all(&str_bytes)?;
        //write null terminator
        self.write_all(&[0])?;
        Ok(str_bytes.len() + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_u8() {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_u8(5).unwrap();

        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0], 5);
    }

    #[test]
    fn write_u32() {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_u32(278).unwrap();

        assert_eq!(buffer.len(), 4);
        assert_eq!(&buffer, &[22, 1, 0, 0]);
    }

    #[test]
    fn write_u64() {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_u64(278).unwrap();

        assert_eq!(buffer.len(), 8);
        assert_eq!(&buffer, &[22, 1, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn write_c_string() {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.write_c_string("Hello").unwrap();

        assert_eq!(buffer.len(), 6);
        assert_eq!(&buffer, &[b'H', b'e', b'l', b'l', b'o', 0]);
    }

    #[test]
    fn read_u8() {
        let mut memory: &[u8] = &[22, 12];
        let (len, val) = memory.read_u8().unwrap();
        assert_eq!(len, 1);
        assert_eq!(val, 22);
    }
    #[test]
    fn read_u32() {
        let mut memory: &[u8] = &[22, 1, 0, 0];
        let (len, val) = memory.read_u32().unwrap();
        assert_eq!(len, 4);
        assert_eq!(val, 278);
    }
    #[test]
    fn read_u64() {
        let mut memory: &[u8] = &[22, 1, 0, 0, 0, 0, 0, 0];
        let (len, val) = memory.read_u64().unwrap();
        assert_eq!(len, 8);
        assert_eq!(val, 278);
    }
    #[test]
    fn read_c_string() {
        let mut memory: &[u8] = b"Hello\0";
        let (len, val) = memory.read_c_string().unwrap();
        assert_eq!(len, 6);
        assert_eq!(val, "Hello");
    }
}
