import socket
import time

signal_server = ('0.0.0.0', 5000)

def send_data(sock, data):
    payload = b'\x01' + len(data).to_bytes(2, 'big') + data
    sock.sendto(payload, signal_server)


def recv_data(sock):
    sock.sendto(b'\x02', signal_server)
    result = []
    data, remote_ep = sock.recvfrom(2048)
    while (data != b'\xFF\xFF\xFF\xFF'):
        offset = 0
        while (offset < len(data)):
            length = int.from_bytes(data[offset:(offset+2)], 'big')
            if length == 0:
                break
            offset += 2
            result.append(data[offset:(offset+length)])
            offset += length

        data, remote_ep = sock.recvfrom(2048)
    
    return result


def create_socket():
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind(('0.0.0.0', 0))
    return sock


for _ in range(13):
    send_data(create_socket(), b'A'*800)
    time.sleep(0.01)

r = recv_data(create_socket())
print("Response length:", len(r))

for i in r:
    if i != b'A'*800:
        print('BROKEN DATA:', i)
