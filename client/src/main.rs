use std::net::UdpSocket;

/// webrtc-unreliable is using a raw UDP socket under the hood
/// but because of the way that it does client tracking it won't work seamlessly without
/// an sdp

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let remote_addr = std::net::SocketAddr::new([127, 0, 0, 1].into(), 3031);
    let local_addr = if remote_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(local_addr)?;
    const MAX_DATAGRAM_SIZE: usize = 65_507;
    socket.connect(&remote_addr)?;

    let mut count = 0;
    loop {
        let input = format!("hello from udp {}", count);
        let size = socket.send(input.as_bytes())?;
        println!("sent {} bytes", size);
        assert_eq!(size, input.len());
        let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
        let len = socket.recv(&mut data)?;
        let s = String::from_utf8_lossy(&data[..len]);
        println!("Received {} bytes: {:?}", len, s);
        assert_eq!(input, s);

        count += 1;
        std::thread::sleep(std::time::Duration::from_secs_f32(0.2));
    }
}
