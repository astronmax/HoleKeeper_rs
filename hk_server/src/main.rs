use std::{
    collections::VecDeque,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
};

use std::net::UdpSocket;

type BinStruct = Vec<u8>;
type DataType = Arc<Mutex<VecDeque<BinStruct>>>;

struct Server {
    address: SocketAddr,
    data: DataType,
}

impl Server {
    const BUFFER_SIZE: usize = 2048;

    pub fn new(address: &str) -> Result<Self, std::io::Error> {
        let addr = match SocketAddr::from_str(address) {
            Ok(v) => v,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                ))
            }
        };

        Ok(Self {
            address: addr,
            data: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    pub async fn start(&self) -> Result<(), std::io::Error> {
        {
            let data_clone = self.data.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                data_clone.lock().unwrap().pop_front();
            });
        }

        let socket = UdpSocket::bind(self.address)?;
        let socket_ptr = Arc::new(socket);

        loop {
            let mut buf = [0_u8; Server::BUFFER_SIZE];
            let (_, from_addr) = socket_ptr.recv_from(&mut buf)?;

            let socket_clone = socket_ptr.clone();
            let data_clone = self.data.clone();
            tokio::spawn(async move {
                match buf[0] {
                    0x01 => Server::process_add(buf, data_clone),
                    0x02 => Server::process_get(socket_clone, from_addr, data_clone),
                    _ => return,
                };
            });
        }
    }

    fn process_add(buf: [u8; Server::BUFFER_SIZE], data_ptr: DataType) {
        // TODO: size of structure (first two bytes)
        let binary_struct = buf[1..].to_vec();
        data_ptr.lock().unwrap().push_back(binary_struct);
    }

    fn process_get(socket: Arc<UdpSocket>, from_addr: SocketAddr, data_ptr: DataType) {
        let data = data_ptr.lock().unwrap();
        let mut response = [0_u8; Server::BUFFER_SIZE];

        let mut offset: usize = 0;
        let mut empty = true;
        for frame in data.iter() {
            if offset + frame.len() + 2 > response.len() {
                // response is full, send package
                socket
                    .send_to(&response, from_addr)
                    .expect("Can't send response");
                empty = true;
            } else {
                // add frame size
                response[offset] = (frame.len() >> 8_usize) as u8;
                response[offset + 1] = (frame.len() & 0xFF_usize) as u8;
                offset += 2;

                // add frame data
                response[offset..(offset + frame.len())].copy_from_slice(frame.as_slice());
                offset += frame.len();

                empty = false;
            }
        }

        if !empty {
            socket
                .send_to(&response, from_addr)
                .expect("Can't send response");
        }

        // no more packages
        socket
            .send_to(&[0xFF, 0xFF, 0xFF, 0xFF], from_addr)
            .expect("Can't send response");
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let server = Server::new("0.0.0.0:5000").unwrap();
    server.start().await?;

    Ok(())
}
