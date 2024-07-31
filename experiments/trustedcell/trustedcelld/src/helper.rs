use crate::access::{AccessVector, Decision};
use anyhow::anyhow;
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::{mpsc, oneshot, RwLock},
};

pub struct HelperHub {
    helpers: Arc<RwLock<HashMap<libc::uid_t, Arc<Helper>>>>,
}
impl HelperHub {
    pub fn listen(path: &Path) -> std::io::Result<Self> {
        let listener = UnixListener::bind(path)?;
        let helpers: Arc<RwLock<HashMap<u32, Arc<Helper>>>> = Arc::default();

        HelperHubImpl {
            helpers: helpers.clone(),
            listener,
        }
        .start();

        Ok(Self { helpers })
    }

    pub async fn decide(&self, access_vector: &AccessVector) -> anyhow::Result<Decision> {
        let lock = self.helpers.read().await;
        let helper = Arc::clone(
            lock.get(&access_vector.subject.uid)
                .ok_or_else(|| anyhow!("no such helper"))?,
        );
        let boxed = Box::new(access_vector.clone());
        match helper.decide(boxed).await {
            Ok(val) => Ok(val),
            Err(err) => {
                drop(lock);
                let mut lock = self.helpers.write().await;
                if let Some(x) = lock.get(&access_vector.subject.uid) {
                    if Arc::ptr_eq(x, &helper) {
                        lock.remove(&access_vector.subject.uid);
                    }
                }
                Err(err)
            }
        }
    }
}

struct Helper {
    tx: mpsc::Sender<Command>,
}
impl Helper {
    async fn decide(&self, av: Box<AccessVector>) -> anyhow::Result<Decision> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Command::AskForPermission(av, tx)).await?;
        Ok(rx.await?)
    }
}

struct HelperHubImpl {
    helpers: Arc<RwLock<HashMap<libc::uid_t, Arc<Helper>>>>,
    listener: UnixListener,
}
impl HelperHubImpl {
    fn start(self) {
        tokio::spawn(async move {
            _ = self.run();
        });
    }

    async fn run(self) -> anyhow::Result<()> {
        while let Ok((client, _)) = self.listener.accept().await {
            let Ok(cred) = client.peer_cred() else {
                continue;
            };
            let (tx, rx) = mpsc::channel(16);
            HelperImpl {
                stream: BufReader::new(client),
                rx,
            }
            .start();
            let helper = Helper { tx };
            self.helpers
                .write()
                .await
                .insert(cred.uid(), Arc::new(helper));
        }
        Ok(())
    }
}

struct HelperImpl {
    stream: BufReader<UnixStream>,
    rx: mpsc::Receiver<Command>,
}
impl HelperImpl {
    fn start(self) {
        tokio::spawn(async move {
            _ = self.run();
        });
    }

    async fn run(mut self) -> anyhow::Result<()> {
        let mut buf = String::with_capacity(4);
        while let Some(x) = self.rx.recv().await {
            match x {
                Command::AskForPermission(av, chan) => {
                    self.stream
                        .write_all(
                            format!(
                                "{} {} {} {}",
                                av.subject.cell, av.object.category, av.object.owner, av.action
                            )
                            .as_bytes(),
                        )
                        .await?;
                    buf.clear();
                    self.stream.read_line(&mut buf).await?;
                    let mut splited = buf.split_whitespace();
                    let allowed = splited
                        .next()
                        .ok_or_else(|| anyhow!("corrupt response"))?
                        .parse::<u32>()?
                        != 0;
                    let cachable = splited
                        .next()
                        .ok_or_else(|| anyhow!("corrupt response"))?
                        .parse::<u32>()?
                        != 0;
                    let decision = match allowed {
                        true => match cachable {
                            true => Decision::Allow,
                            false => Decision::AllowOnce,
                        },
                        false => match cachable {
                            true => Decision::Deny,
                            false => Decision::DenyOnce,
                        },
                    };
                    _ = chan.send(decision);
                }
            }
        }
        Ok(())
    }
}

enum Command {
    AskForPermission(Box<AccessVector>, oneshot::Sender<Decision>),
}
