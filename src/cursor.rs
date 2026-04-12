use anyhow::{bail, Result};

pub(crate) struct Cursor<'a> {
    data:  &'a [u8],
    pos:   usize,
    label: &'static str,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8], label: &'static str) -> Self {
        Self { data, pos: 0, label }
    }

    pub fn set_pos(&mut self, pos: usize) { self.pos = pos; }

    pub fn pos(&self) -> usize { self.pos }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn need(&self, n: usize) -> Result<()> {
        if self.pos + n > self.data.len() {
            bail!(
                "{} truncated at offset {} (need {} more bytes)",
                self.label, self.pos, n
            )
        } else {
            Ok(())
        }
    }

    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.need(n)?;
        self.pos += n;
        Ok(())
    }

    pub fn u8(&mut self) -> Result<u8> {
        self.need(1)?;
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn u16(&mut self) -> Result<u16> {
        self.need(2)?;
        let v = u16::from_le_bytes(self.data[self.pos..self.pos + 2].try_into().unwrap());
        self.pos += 2;
        Ok(v)
    }

    pub fn u32(&mut self) -> Result<u32> {
        self.need(4)?;
        let v = u32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }

    pub fn i32(&mut self) -> Result<i32> {
        self.need(4)?;
        let v = i32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }

    pub fn f32(&mut self) -> Result<f32> {
        self.need(4)?;
        let v = f32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }

    /// Read `len` raw bytes.
    pub fn bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        self.need(len)?;
        let v = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(v)
    }

    /// uint16-length-prefixed UTF-8 string (lossy). Growtopia wire encoding.
    pub fn plain_string(&mut self) -> Result<String> {
        let len = self.u16()? as usize;
        self.need(len)?;
        let s = String::from_utf8_lossy(&self.data[self.pos..self.pos + len]).into_owned();
        self.pos += len;
        Ok(s)
    }

    /// Raw bytes as UTF-8 string (lossy), no length prefix.
    pub fn string_raw(&mut self, len: usize) -> Result<String> {
        self.need(len)?;
        let s = String::from_utf8_lossy(&self.data[self.pos..self.pos + len]).into_owned();
        self.pos += len;
        Ok(s)
    }

    /// uint16-length-prefixed XOR-decrypted string.
    /// `key_start`: byte offset into `key` to begin XOR (wraps around).
    pub fn xor_string(&mut self, key: &[u8], key_start: usize) -> Result<String> {
        let len = self.u16()? as usize;
        self.need(len)?;
        let bytes: Vec<u8> = self.data[self.pos..self.pos + len]
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ key[(key_start + i) % key.len()])
            .collect();
        self.pos += len;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}
