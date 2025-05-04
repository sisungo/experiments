//! Local data management.

pub mod dotenv;
pub mod temp;

pub use temp::Temp;

#[derive(Debug)]
pub struct LocalData {
    pub temp: Temp,
}
impl LocalData {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self { temp: Temp::new()? })
    }
}
