use std::{
    collections::HashMap,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
};

use std::net::UdpSocket;

type BinStruct = Vec<u8>;
type ClientsList = HashMap<SocketAddr, (BinStruct, std::time::Duration)>;

struct Server {
    address: SocketAddr,
    clients: Arc<Mutex<ClientsList>>,
}

fn get_time_now() -> std::time::Duration {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
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
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> Result<(), std::io::Error> {
        {
            let pause = std::time::Duration::from_secs(30);
            let clients_clone = self.clients.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(pause);
                let mut clients = clients_clone.lock().unwrap();
                let time_now = get_time_now();
                let mut clients_to_remove = Vec::new();
                for client in clients.keys() {
                    let (_, last_request) = clients.get(client).unwrap();
                    if time_now - *last_request > pause {
                        clients_to_remove.push(client.clone());
                    }
                }

                for c in clients_to_remove {
                    println!("[INFO] Client {} has been removed", c);
                    clients.remove(&c);
                    println!("[INFO] Online: {}", clients.len());
                }
            });
        }

        let socket = UdpSocket::bind(self.address)?;
        let socket_ptr = Arc::new(socket);

        loop {
            let mut buf = [0_u8; Server::BUFFER_SIZE];
            let (_, from_addr) = socket_ptr.recv_from(&mut buf)?;

            let socket_clone = socket_ptr.clone();
            let clients_mtx = self.clients.clone();
            tokio::spawn(async move {
                match buf[0] {
                    0x01 => Server::process_add(buf, from_addr, clients_mtx),
                    0x02 => Server::process_get(socket_clone, from_addr, clients_mtx),
                    _ => return,
                };
            });
        }
    }

    fn process_add(
        buf: [u8; Server::BUFFER_SIZE],
        addr: SocketAddr,
        clients_mtx: Arc<Mutex<ClientsList>>,
    ) {
        let length = ((buf[1] as usize) << 8) | (buf[2] as usize);
        if length > 0 && length < Server::BUFFER_SIZE - 3 {
            let binary_struct = buf[3..(3 + length)].to_vec();
            clients_mtx
                .lock()
                .unwrap()
                .insert(addr, (binary_struct, get_time_now()));
        }
    }

    fn process_get(socket: Arc<UdpSocket>, addr: SocketAddr, clients_mtx: Arc<Mutex<ClientsList>>) {
        let clients = clients_mtx.lock().unwrap();
        let mut response = [0_u8; Server::BUFFER_SIZE];

        let mut offset: usize = 0;
        for (frame, _) in clients.values() {
            if offset + frame.len() + 2 > response.len() {
                socket
                    .send_to(&response, addr)
                    .expect("Can't send response");

                response.fill(0);
                offset = 0;
            }
            response[offset] = (frame.len() >> 8_usize) as u8;
            response[offset + 1] = (frame.len() & 0xFF_usize) as u8;
            offset += 2;

            response[offset..(offset + frame.len())].copy_from_slice(frame.as_slice());
            offset += frame.len();
        }

        socket
            .send_to(&response, addr)
            .expect("Can't send response");

        socket
            .send_to(&[0xFF, 0xFF, 0xFF, 0xFF], addr)
            .expect("Can't send response");
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let server = Server::new("0.0.0.0:5000").unwrap();
    server.start().await?;

    Ok(())
}
