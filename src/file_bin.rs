use std::error::Error;
use std::io::prelude::*;
use std::io::Write;
use std::io::SeekFrom;
use std::fs::File;

pub struct BinFile {
    offset: u8,
    buffer: u8,
    file: std::fs::File,

    pub size: Option<usize>,
    pub path: String,
}

impl BinFile {
    pub fn create(path: &String) -> Result<BinFile, Box<dyn Error>> {
        Ok(BinFile {
            offset: 0u8,
            buffer: 0u8,
            file: std::fs::File::create(path)?,
            size: None,
            path: path.clone(),
        })
    }

    pub fn open(path: &String) -> Result<BinFile, Box<dyn Error>> {
        Ok(BinFile {
            offset: 0u8,
            buffer: 0u8,
            file: File::open(path)?,
            size: None,
            path: path.clone(),
        })
    }

    pub fn tell(&mut self) -> Result<u64, Box<dyn Error>> {
        Ok(self.file.seek(SeekFrom::Current(0))?)
    }

    pub fn read_bit(&mut self) -> Result<bool, Box<dyn Error>> {
        // If we need a new byte, read it from the file
        if self.offset == 0 {
            let mut t = [0;1];
            self.file.read_exact(&mut t)?;
            self.buffer = t[0];
        }
        // Extract the bit as boolean
        let ret = (self.buffer & (1 << (7 - self.offset))) != 0;
        // Increment modulo 8
        self.offset = (self.offset +1) % 8;
        // Return the bit as boolean
        Ok(ret)
    }

    pub fn read_byte(&mut self) -> Result<u8, Box<dyn Error>> {
        let mut ret: u8 = 0u8;
        for _ in 0..8 {
            match self.read_bit()? {
                true => ret = (ret << 1) | 1,
                false => ret <<= 1
            };
        }
        print!("{}", ret);
        Ok(ret)
    }

    pub fn read_bytes(&mut self, nb: usize) -> Result<Box<[u8]>, Box<dyn Error>> {
        // Just loop nb times and call self.read_byte()
        let mut ret = vec![0u8; nb];
        for i in 0..nb {
            ret[i] = self.read_byte()?;
        }
        Ok(ret.into_boxed_slice())
    }

    pub fn write_bit(&mut self, bit: bool) -> Result<bool, Box<dyn Error>> {
        match bit {
            true => {
                self.buffer <<= 1;
                self.buffer |= 1;
            },
            false => self.buffer <<= 1
        };

        self.offset += 1;

        if self.offset == 8 {
            self.file.write_all(&[self.buffer])?;
            self.offset = 0;
            self.buffer = 0;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<(), Box<dyn Error>> {
        // Just loop over each bit of byte and call self.write_bit()
        for i in 0u8..8u8 {
            match ((byte >> (7 - i)) & 1) != 0 {
                true => self.write_bit(true)?,
                false => self.write_bit(false)?,
            };
        }
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
        // Just loop through bytes and call self.write_byte()
        for b in bytes {
            self.write_byte(*b)?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        // Writes 0-bit untill the offset is 0
        while self.offset != 0 {
            self.write_bit(false)?;
        }
        Ok(())
    }

    pub fn read_size(&mut self) -> Result<usize, Box<dyn Error>> {
        let mut buf = [0u8; 8];
        let n = self.file.read(&mut buf)?;

        if n != 8 {
            panic!("[+] Wrong file format");
        }
        self.size = Some(usize::from_le_bytes(buf));
        Ok(self.size.unwrap())
    }

}