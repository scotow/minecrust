use std::io::{Read, Write};

use crate::types::*;
use crate::error::Result;

pub trait ReadExtension: Read + Sized {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buff = [0; 1];
        self.read_exact(&mut buff)?;
        Ok(buff[0])
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buff = [0; 2];
        self.read_exact(&mut buff)?;
        Ok(((buff[0] as u16) << 8) | buff[1] as u16)
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut buff = [0; 8];
        self.read_exact(&mut buff)?;
        Ok(u64::from_be_bytes(buff))
    }

    fn read_i8(&mut self) -> Result<i8> {
        self.read_u8().map(|n| n as i8)
    }

    fn read_i16(&mut self) -> Result<i16> {
        self.read_u16().map(|n| n as i16)
    }

    fn read_i64(&mut self) -> Result<i64> {
        self.read_u64().map(|n| n as i64)
    }

    fn read_var_int(&mut self) -> Result<VarInt> {
        VarInt::parse(self)
    }

    fn read_string(&mut self) -> Result<String> {
        String::parse(self)
    }
}

impl<R: Read> ReadExtension for R {}

pub trait WriteExtension: Write + Sized {
    fn write_u8(&mut self, n: u8) -> Result<()> {
        let buff = [n; 1];
        self.write_all(&buff)?;
        Ok(())
    }

    fn write_i64(&mut self, n: i64) -> Result<()> {
        self.write_all(&n.to_be_bytes())?;
        Ok(())
    }

    fn write_var_int(&mut self, n: VarInt) -> Result<()> {
        n.write(self)
    }

    fn write_string(&mut self, s: &String) -> Result<()> {
        s.write(self)
    }
}

impl<W: Write> WriteExtension for W {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::stream::ReadExtension;

    #[test]
    fn read_u8() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xDE, 0xAD]);
        assert_eq!(buffer.read_u8()?, 0xDE);
        assert_eq!(buffer.read_u8()?, 0xAD);
        assert!(buffer.read_u8().is_err());
        Ok(())
    }

    #[test]
    fn read_u16() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xDE, 0xAD, 0xBE, 0xEF, 0x42]);
        assert_eq!(buffer.read_u16()?, 0xDEAD);
        assert_eq!(buffer.read_u16()?, 0xBEEF);
        assert!(buffer.read_u16().is_err());
        Ok(())
    }

    #[test]
    fn read_i8() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xFF, 0x7F]);
        assert_eq!(buffer.read_i8()?, -1);
        assert_eq!(buffer.read_i8()?, 127);
        assert!(buffer.read_i8().is_err());
        Ok(())
    }
}