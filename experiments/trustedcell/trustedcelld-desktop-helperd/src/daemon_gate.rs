use std::io::{Read, Write};

pub struct MessageProto<T>(T);
impl<T> MessageProto<T>
where T: Read
{
    pub fn recv(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut len = [0u8; size_of::<u32>()];
        self.0.read_exact(&mut len)?;
        let len = u32::from_le_bytes(len) as usize;
        let mut buf = vec![0u8; len];
        self.0.read_exact(&mut buf)?;
        Ok(buf)
    }
}
impl<T> MessageProto<T>
where T: Write
{
    pub fn send(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        self.0.write_all(&(buf.len() as u32).to_le_bytes())?;
        self.0.write_all(buf)?;
        Ok(())
    }
}
impl<T> From<T> for MessageProto<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}