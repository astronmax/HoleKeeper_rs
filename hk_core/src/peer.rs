use std::{
    io::{Error, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

#[derive(Clone)]
pub struct Peer {
    nickname: String,
    address: SocketAddr,
    turn_use: bool,
    turn_server: SocketAddr,
}

impl Peer {
    const NICKNAME_LEN: usize = 200;
    const ADDRESS_LEN: usize = 6;
    const LEN: usize = Peer::NICKNAME_LEN + (Peer::ADDRESS_LEN * 2) + 1;

    pub fn new() -> Self {
        Self {
            nickname: "".to_string(),
            address: SocketAddr::from_str("0.0.0.0:0").unwrap(),
            turn_use: false,
            turn_server: SocketAddr::from_str("0.0.0.0:0").unwrap(),
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut packed = Vec::<u8>::new();
        let mut packed_nickname = self.nickname.as_bytes().to_vec();
        packed_nickname.append(&mut vec![0_u8; Peer::NICKNAME_LEN - packed_nickname.len()]);
        packed.append(&mut packed_nickname);

        let pack_address = |addr: SocketAddr| {
            let mut packed_addr = Vec::<u8>::new();
            match addr.ip() {
                IpAddr::V4(ip) => {
                    for i in ip.octets() {
                        packed_addr.push(i);
                    }
                }
                IpAddr::V6(_) => panic!("IPv6 addresses are not supported"),
            }
            packed_addr.push(((addr.port() >> 8) & 0xFF) as u8);
            packed_addr.push((addr.port() & 0xFF) as u8);

            return packed_addr;
        };

        packed.append(&mut pack_address(self.address));
        packed.push(self.turn_use as u8);
        packed.append(&mut pack_address(self.turn_server));

        packed
    }

    pub fn unpack(raw_data: Vec<u8>) -> Result<Self, Error> {
        if raw_data.len() != Peer::LEN {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Packed Peer size must be {} bytes", Peer::LEN),
            ));
        }

        let nickname = match std::str::from_utf8(&raw_data[0..Peer::NICKNAME_LEN]) {
            Ok(v) => v.replace('\0', ""),
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.to_string())),
        };

        let unpack_address = |packed_addr: &[u8]| {
            if packed_addr.len() != 6 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Packed address size must be {} bytes", Peer::ADDRESS_LEN),
                ));
            }
            Ok(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(
                    packed_addr[0],
                    packed_addr[1],
                    packed_addr[2],
                    packed_addr[3],
                )),
                ((packed_addr[4] as u16) << 8_u8) | (packed_addr[5] as u16),
            ))
        };

        let offset = Peer::NICKNAME_LEN + Peer::ADDRESS_LEN;
        let address = match unpack_address(&raw_data[Peer::NICKNAME_LEN..offset]) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let turn_use = raw_data[offset] != 0;
        let turn_address = match unpack_address(&raw_data[(offset + 1)..]) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        Ok(Self {
            nickname: nickname,
            address: address,
            turn_use: turn_use,
            turn_server: turn_address,
        })
    }

    pub fn set_nickname(&mut self, nickname: String) -> Result<(), Error> {
        if nickname.len() > Peer::NICKNAME_LEN {
            return Err(Error::new(ErrorKind::InvalidData, "Nickname is too long"));
        }

        self.nickname = nickname;
        Ok(())
    }

    pub fn get_nickname(&self) -> &String {
        &self.nickname
    }

    pub fn set_address(&mut self, addr: SocketAddr) {
        self.address = addr;
    }

    pub fn get_address(&self) -> SocketAddr {
        self.address
    }

    pub fn set_turn_server(&mut self, addr: SocketAddr) {
        self.turn_use = true;
        self.turn_server = addr;
    }

    pub fn disable_turn_server(&mut self) {
        self.turn_use = false;
    }

    pub fn get_turn_server(&self) -> SocketAddr {
        self.turn_server
    }

    pub fn using_turn(&self) -> bool {
        self.turn_use
    }
}
