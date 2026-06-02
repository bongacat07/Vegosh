use rustix::net::{
    socket, bind, listen, acceptfrom_with,
    AddressFamily, SocketType, ipproto, SocketFlags
};
use rustix::fd::OwnedFd;
use rustix::io::{read, write};
use std::net::{Ipv4Addr, SocketAddrV4};
use crate::protocol::parser;
use crate::protocol::KVStore;


pub fn initialise_server() -> rustix::io::Result<OwnedFd> {
    let sockfd = socket(
        AddressFamily::INET,
        SocketType::STREAM,
        Some(ipproto::TCP),
    )?;

    bind(&sockfd, &SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080))?;

    listen(&sockfd, 128)?;
    let mut store: KVStore = KVStore::new();
    loop {
        let (conn, peer_addr) = acceptfrom_with(&sockfd, SocketFlags::CLOEXEC)?;

        println!("New connection from {:?}", peer_addr);

        parser(&conn,&mut store );
    }
}
