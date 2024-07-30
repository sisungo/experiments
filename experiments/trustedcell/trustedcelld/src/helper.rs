use crate::access::{AccessVector, Decision};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{net::UnixListener, sync::RwLock};

pub struct HelperHub {
    helpers: RwLock<HashMap<libc::uid_t, Arc<Helper>>>,
}
impl HelperHub {
    pub fn listen(path: &Path) -> std::io::Result<Self> {
        let listener = UnixListener::bind(path)?;

        HelperHubImpl { listener }.start();

        Ok(Self {
            helpers: RwLock::default(),
        })
    }

    pub async fn decide(&self, access_vector: &AccessVector) -> anyhow::Result<Decision> {
        todo!()
    }
}

pub struct Helper {}

struct HelperHubImpl {
    listener: UnixListener,
}
impl HelperHubImpl {
    fn start(self) {
        tokio::spawn(async move {
            _ = self.run();
        });
    }

    async fn run(self) -> anyhow::Result<()> {
        while let Ok(client) = self.listener.accept().await {}
        Ok(())
    }
}
