use std::io::{Read, Write};
use anyhow::anyhow;
use crate::{AccessVector, Decision, Object};

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

pub fn access_vector_of(s: &str) -> anyhow::Result<AccessVector> {
    let mut splited = s.split(' ');
        let subject_cell = splited
            .next()
            .ok_or_else(|| anyhow!("incomplete access vector"))?;
        let object_category = splited
            .next()
            .ok_or_else(|| anyhow!("incomplete access vector"))?;
        let object_owner = splited
            .next()
            .ok_or_else(|| anyhow!("incomplete access vector"))?;
        let action = splited
            .next()
            .ok_or_else(|| anyhow!("incomplete access vector"))?;
        Ok(AccessVector {
            subject_cell: subject_cell.into(),
            object: Object {
                category: object_category.into(),
                owner: object_owner.into(),
            },
            action: action.parse().unwrap(),
        })
}

pub fn of_decision(decision: Decision) -> &'static [u8; 3] {
    match decision {
        Decision::Allow => b"1 1",
        Decision::AllowOnce => b"1 0",
        Decision::Deny => b"0 1",
        Decision::DenyOnce => b"0 0",
    }
}
