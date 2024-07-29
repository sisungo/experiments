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
        Ok(())
    }
}
