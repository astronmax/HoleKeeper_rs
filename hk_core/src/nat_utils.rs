use std::{net::SocketAddr, str::FromStr, sync::Arc};

use tokio::net::UdpSocket;
use webrtc::stun::{
    agent::TransactionId,
    message::{Getter, BINDING_REQUEST},
    xoraddr::XorMappedAddress,
    Error,
};

pub enum NatType {
    Common,
    Symmetric,
}

const STUN_SERVERS: [&str; 2] = ["108.177.14.127:19302", "216.93.246.18:3478"];

pub async fn get_remote_address(
    socket: Arc<UdpSocket>,
    stun_addr: SocketAddr,
) -> Result<SocketAddr, Error> {
    socket.connect(stun_addr).await?;

    let mut client = webrtc::stun::client::ClientBuilder::new()
        .with_conn(socket)
        .build()?;

    let (handler_tx, mut handler_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut msg = webrtc::stun::message::Message::new();
    msg.build(&[Box::<TransactionId>::default(), Box::new(BINDING_REQUEST)])?;
    client.send(&msg, Some(Arc::new(handler_tx))).await?;

    let event = handler_rx.recv().await.unwrap();
    let msg = event.event_body?;
    let mut xor_addr = XorMappedAddress::default();
    xor_addr.get_from(&msg)?;

    Ok(SocketAddr::new(xor_addr.ip, xor_addr.port))
}

pub async fn get_nat_type() -> Result<NatType, Error> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let socket_ptr = Arc::new(socket);

    let addr_1 = get_remote_address(
        socket_ptr.clone(),
        SocketAddr::from_str(STUN_SERVERS[0]).unwrap(),
    )
    .await?;

    let addr_2 = get_remote_address(
        socket_ptr.clone(),
        SocketAddr::from_str(STUN_SERVERS[0]).unwrap(),
    )
    .await?;

    if addr_1 != addr_2 {
        return Ok(NatType::Symmetric);
    }

    Ok(NatType::Common)
}
