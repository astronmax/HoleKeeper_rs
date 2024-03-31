use std::{net::SocketAddr, str::FromStr};

use super::peer::Peer;

#[test]
fn pack_peer_test() {
    let mut peer = Peer::new();
    peer.set_nickname("username".to_string()).unwrap();
    peer.set_address(SocketAddr::from_str("95.27.168.74:33124").unwrap());
    peer.set_turn_server(SocketAddr::from_str("102.77.198.34:34123").unwrap());

    let packed_peer = peer.pack();
    let peer_from_bytes = Peer::unpack(packed_peer).unwrap();

    assert_eq!(peer_from_bytes.get_nickname(), peer.get_nickname());
    assert_eq!(peer_from_bytes.get_address(), peer.get_address());
    assert_eq!(peer_from_bytes.using_turn(), peer.using_turn());
    assert_eq!(peer_from_bytes.get_turn_server(), peer.get_turn_server());
}
