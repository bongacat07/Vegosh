use rustix::net::{
    socket, bind, listen, acceptfrom_with,
    AddressFamily, SocketType, ipproto, SocketFlags
};
use rustix::fd::OwnedFd;
use std::net::{Ipv4Addr, SocketAddrV4};
use vegosh_db::protocol::*;


fn main() -> rustix::io::Result<()> {
    let sockfd = socket(
        AddressFamily::INET,
        SocketType::STREAM,
        Some(ipproto::TCP),
    )?;

    bind(&sockfd, &SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080))?;

    listen(&sockfd, 128)?;
    let mut store: KVStore = KVStore::new();
    loop {
        let (conn, peer_addr) = acceptfrom_with(&sockfd, SocketFlags::empty())?;
        println!("New connection from {:?}", peer_addr);
        loop {
            if let Err(_) = parser(&conn, &mut store) {
                break;
            }
        }
    }
}
