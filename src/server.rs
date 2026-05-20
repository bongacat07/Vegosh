use rustix::net::{
    socket, bind_v4, listen, accept,
    AddressFamily, SocketType, SocketAddrV4,
    ipproto,
};
use rustix::io::read;
use rustix::io::write;
use std::net::Ipv4Addr;

fn main() {
    // 1. Create a socket (like asking the OS: "give me a communication endpoint")
    let sock = socket(
        AddressFamily::INET,       // IPv4
        SocketType::STREAM,        // TCP (stream-based, not UDP)
        Some(ipproto::TCP),        // explicitly TCP
    ).expect("failed to create socket");

    // 2. Bind to address 127.0.0.1:8080
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    bind_v4(&sock, &addr).expect("failed to bind");

    // 3. Start listening (backlog = 10 means queue up to 10 pending connections)
    listen(&sock, 10).expect("failed to listen");

    println!("Server listening on 127.0.0.1:8080");

    loop {
        // 4. Accept a connection — blocks here until someone connects
        let (conn, _peer_addr) = accept(&sock).expect("failed to accept");

        // 5. Read what they sent
        let mut buf = [0u8; 1024];
        let n = read(&conn, &mut buf).expect("failed to read");

        println!("Received: {}", String::from_utf8_lossy(&buf[..n]));

        // 6. Write a response back
        write(&conn, b"Hello from rustix server!\n").expect("failed to write");

        // conn drops here, closing the connection
    }
}
