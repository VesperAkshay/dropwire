use crate::error::DropWireError;
use crate::types::ChannelId;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;

pub struct DiscoveryService;

impl DiscoveryService {
    pub async fn announce(
        channel: &ChannelId,
        duration: Duration,
        port: u16,
    ) -> Result<(), DropWireError> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(DropWireError::Io)?;
        socket.set_broadcast(true).map_err(DropWireError::Io)?;
        let multicast_addr: SocketAddr = "239.255.255.250:9999".parse().unwrap();

        let msg = format!("DROPWIRE/1.0 DISCOVER {} PORT {}\n", channel.0, port);
        let start = tokio::time::Instant::now();

        while start.elapsed() < duration {
            let _ = socket.send_to(msg.as_bytes(), multicast_addr).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    pub async fn find_peer(
        channel: &ChannelId,
        timeout: Duration,
    ) -> Result<SocketAddr, DropWireError> {
        let socket = UdpSocket::bind("0.0.0.0:9999")
            .await
            .map_err(DropWireError::Io)?;

        let multi_addr: Ipv4Addr = "239.255.255.250".parse().unwrap();
        let interface = Ipv4Addr::new(0, 0, 0, 0);
        socket
            .join_multicast_v4(multi_addr, interface)
            .map_err(DropWireError::Io)?;

        let expected_prefix = format!("DROPWIRE/1.0 DISCOVER {} PORT ", channel.0);
        let mut buf = [0u8; 1024];

        let res = tokio::time::timeout(timeout, async {
            loop {
                let (len, src) = socket
                    .recv_from(&mut buf)
                    .await
                    .map_err(DropWireError::Io)?;
                    
                let received_str = String::from_utf8_lossy(&buf[..len]);
                if received_str.starts_with(&expected_prefix) {
                    let port_str = received_str.trim().split(' ').last().unwrap_or("0");
                    if let Ok(port) = port_str.parse::<u16>() {
                        let mut peer_addr = src;
                        peer_addr.set_port(port);
                        return Ok::<_, DropWireError>(peer_addr);
                    }
                }
            }
        })
        .await;

        match res {
            Ok(Ok(src)) => Ok(src),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(DropWireError::Network("Timeout".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Since find_peer binds to a fixed port (9999), tests must be run with --test-threads=1

    #[tokio::test]
    async fn test_discovery_success() {
        let chan = ChannelId("test_chan1".into());

        let find_fut = tokio::spawn({
            let chan = chan.clone();
            async move { DiscoveryService::find_peer(&chan, Duration::from_secs(2)).await }
        });

        tokio::time::sleep(Duration::from_millis(50)).await; // Let receiver bind

        let announce_fut = tokio::spawn({
            let chan = chan.clone();
            async move { DiscoveryService::announce(&chan, Duration::from_millis(200), 8080).await }
        });

        let (find_res, ann_res) = tokio::join!(find_fut, announce_fut);
        let src_addr = find_res.unwrap().unwrap();
        ann_res.unwrap().unwrap(); // Announce completed successfully

        assert_eq!(src_addr.port(), 8080);
    }

    #[tokio::test]
    async fn test_wrong_channel() {
        let find_chan = ChannelId("test_chan2".into());
        let ann_chan = ChannelId("test_chan2_wrong".into());

        let find_fut = tokio::spawn({
            let chan = find_chan.clone();
            async move { DiscoveryService::find_peer(&chan, Duration::from_millis(300)).await }
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        let announce_fut = tokio::spawn({
            let chan = ann_chan.clone();
            async move { DiscoveryService::announce(&chan, Duration::from_millis(100), 8081).await }
        });

        let (find_res, _) = tokio::join!(find_fut, announce_fut);
        let res = find_res.unwrap();

        assert!(matches!(res, Err(DropWireError::Network(_)))); // Timeout
    }

    #[tokio::test]
    async fn test_timeout() {
        let chan = ChannelId("test_chan3".into());
        let res = DiscoveryService::find_peer(&chan, Duration::from_millis(100)).await;
        assert!(matches!(res, Err(DropWireError::Network(_))));
    }

    #[tokio::test]
    async fn test_multiple_announcements() {
        let chan = ChannelId("test_chan4".into());

        let find_fut = tokio::spawn({
            let chan = chan.clone();
            async move { DiscoveryService::find_peer(&chan, Duration::from_secs(2)).await }
        });

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Spawn multiple announcers
        for _ in 0..3 {
            let chan_clone = chan.clone();
            tokio::spawn(async move {
                let _ = DiscoveryService::announce(&chan_clone, Duration::from_millis(200), 8082).await;
            });
        }

        let find_res = find_fut.await.unwrap();
        assert!(find_res.is_ok());
    }
}
