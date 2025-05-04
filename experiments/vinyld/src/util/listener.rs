use crate::local_data::dotenv::*;
use anyhow::anyhow;
use tokio::net::TcpListener;
use url::Url;

/// Returns a listener for the server.
pub async fn listener() -> anyhow::Result<TcpListener> {
    let listen_url = fetch_env::<Url>(NETWORK_LISTEN_URL)?;
    let host = listen_url
        .host()
        .ok_or_else(|| anyhow!("no host in environment `NETWORK_LISTEN_URL`"))?;
    let port = listen_url.port_or_known_default().ok_or_else(|| {
        anyhow!(
            "no port in environment `NETWORK_LISTEN_URL`, possibly the url is in invalid scheme?"
        )
    })?;

    match listen_url.scheme() {
        "http" => Ok(TcpListener::bind(format!("{host}:{port}")).await?),
        "https" => {
            todo!()
        }
        scheme => Err(anyhow!(
            "unknown url scheme `{scheme}` in environment `NETWORK_LISTEN_URL`"
        )),
    }
}
