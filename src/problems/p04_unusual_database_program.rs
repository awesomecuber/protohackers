use std::collections::HashMap;

use color_eyre::Result;
use tokio::net::UdpSocket;
use tracing::{info, instrument};

#[instrument]
pub async fn run_server(ip: &str, port: u32) -> Result<()> {
    let socket = UdpSocket::bind(format!("{ip}:{port}")).await?;
    info!("listening at {ip}:{port}");
    let mut buf = vec![0; 1024];

    let mut store = HashMap::new();
    store.insert("version".to_owned(), "Nico's Store".to_owned());
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        info!("received data");
        let Ok(request) = std::str::from_utf8(&buf[..len]) else {
            continue;
        };
        let request = request.trim();
        if let Some(response) = get_response(request, &mut store) {
            socket.send_to(response.as_bytes(), addr).await?;
        }
    }
}

#[instrument(ret, skip(store))]
fn get_response(request: &str, store: &mut HashMap<String, String>) -> Option<String> {
    let mut parts = request.splitn(2, '=');
    let key = parts.next().unwrap();
    match parts.next() {
        Some(val) => {
            if key != "version" {
                store.insert(key.to_owned(), val.to_owned());
            }
            None
        }
        None => Some(
            store
                .get(key)
                .map(|v| v.to_owned())
                .unwrap_or_else(|| "".to_owned()),
        ),
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::get_response;

    #[test]
    fn test_get_response() {
        let mut store = HashMap::new();
        get_response("foo=bar", &mut store);
        assert_eq!(get_response("foo", &mut store), Some("bar".to_owned()));
        get_response("foo=bar=baz", &mut store);
        assert_eq!(get_response("foo", &mut store), Some("bar=baz".to_owned()));
        get_response("foo=", &mut store);
        assert_eq!(get_response("foo", &mut store), Some("".to_owned()));
        get_response("foo===", &mut store);
        assert_eq!(get_response("foo", &mut store), Some("==".to_owned()));
        get_response("=foo", &mut store);
        assert_eq!(get_response("", &mut store), Some("foo".to_owned()));
    }
}
