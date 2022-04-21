use std::io;
use tokio::net::UdpSocket;

const DAEMON_HEADER: &[u8] = b"{\"format\": \"json\", \"version\": 1}\n";
const DEFAULT_UDP_REMOTE_PORT: u16 = 2000;

pub(crate) struct DaemonClient<S: ClientState> {
    state: S,
}

pub struct Start {
    remote_port: u16,
    remote_addr: String,
}

pub struct Connected {
    sock: UdpSocket,
}

pub struct ConnectedBlocking {
    sock: std::net::UdpSocket,
}

pub(crate) trait ClientState {}
impl ClientState for Start {}
impl ClientState for Connected {}
impl ClientState for ConnectedBlocking {}

impl DaemonClient<Start> {
    pub(crate) fn new(remote_port: u16) -> Self {
        let remote_addr = format!("127.0.0.1:{}", remote_port);
        DaemonClient {
            state: Start {
                remote_port,
                remote_addr,
            },
        }
    }

    pub(crate) async fn connect(&self) -> io::Result<DaemonClient<Connected>> {
        // Let the OS choose an IP and port for us...
        let sock = UdpSocket::bind("0.0.0.0:0").await?;
        sock.connect(self.state.remote_addr.clone()).await?;
        Ok(DaemonClient {
            state: Connected { sock },
        })
    }

    pub(crate) fn connect_blocking(&self) -> io::Result<DaemonClient<ConnectedBlocking>> {
        // Let the OS choose an IP and port for us...
        let sock = std::net::UdpSocket::bind("0.0.0.0:0")?;
        sock.connect(self.state.remote_addr.clone())?;
        Ok(DaemonClient {
            state: ConnectedBlocking { sock },
        })
    }
}

impl Default for DaemonClient<Start> {
    fn default() -> Self {
        DaemonClient::new(DEFAULT_UDP_REMOTE_PORT)
    }
}

impl DaemonClient<Connected> {
    pub(crate) async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let newline = b"\n";
        self.state
            .sock
            .send(&[DAEMON_HEADER, buf, newline].concat())
            .await
    }
}

impl DaemonClient<ConnectedBlocking> {
    pub(crate) fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let newline = b"\n";
        self.state
            .sock
            .send(&[DAEMON_HEADER, buf, newline].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn send_data() {
        let buf = b"{\"hello\": \"world\"}";
        let client: DaemonClient<Start> = Default::default();
        let client = client.connect().await.unwrap();
        let len = client.send(buf).await.unwrap();
        assert_eq!(len, DAEMON_HEADER.len() + buf.len());
    }
}
