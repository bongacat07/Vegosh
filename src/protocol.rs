use rustix::fd::OwnedFd;
use rustix::io::{read, write};

pub const MAX_KEY_LEN: usize = 16;
pub const MAX_VALUE_LEN: usize = 32;
pub const MAX_CONNECTIONS: usize = 10000;

const MAGIC_BYTES: [u8; 2] = [b'v', b'k'];
const SET_KEY:    [u8; 1] = [0x01];
const GET_KEY:    [u8; 1] = [0x02];
const DELETE_KEY: [u8; 1] = [0x03];
const SIZE:       [u8; 1] = [0x04];
const PING:       [u8; 1] = [0x05];
const FLUSH:      [u8; 1] = [0x06];

pub fn readn(fd: &OwnedFd, buf: &mut [u8]) -> rustix::io::Result<usize> {
    let mut total = 0;
    while total < buf.len() {
        match read(fd, &mut buf[total..])? {
            0 => break,
            n => total += n,
        }
    }
    Ok(total)
}

pub fn writen(fd: &OwnedFd, buf: &[u8]) -> rustix::io::Result<()> {
    let mut total = 0;
    while total < buf.len() {
        let n = write(fd, &buf[total..])?;
        total += n;
    }
    Ok(())
}

pub fn parse(fd: &OwnedFd) -> rustix::io::Result<()> {
    // validate magic bytes
    let mut magic = [0u8; 2];
    readn(fd, &mut magic)?;
    if magic != MAGIC_BYTES {
        println!("Invalid client — wrong magic bytes");
        return Ok(());
    }

    // read opcode
    let mut opcode = [0u8; 1];
    readn(fd, &mut opcode)?;

    match opcode {
        SET_KEY    => handle_set(fd)?,
        GET_KEY    => handle_get(fd)?,
        DELETE_KEY => handle_delete(fd)?,
        SIZE       => handle_size(fd)?,
        PING       => handle_ping(fd)?,
        FLUSH      => handle_flush(fd)?,
        _          => println!("Unknown opcode: 0x{:02x}", opcode[0]),
    }

    Ok(())
}

fn handle_set(fd: &OwnedFd) -> rustix::io::Result<()> {
    let mut key = [0u8; MAX_KEY_LEN];
    let mut value = [0u8; MAX_VALUE_LEN];
    readn(fd, &mut key)?;
    readn(fd, &mut value)?;
    println!("SET {:?} = {:?}", &key, &value);
    Ok(())
}

fn handle_get(fd: &OwnedFd) -> rustix::io::Result<()> {
    let mut key = [0u8; MAX_KEY_LEN];
    readn(fd, &mut key)?;
    println!("GET {:?}", &key);
    Ok(())
}

fn handle_delete(fd: &OwnedFd) -> rustix::io::Result<()> {
    let mut key = [0u8; MAX_KEY_LEN];
    readn(fd, &mut key)?;
    println!("DELETE {:?}", &key);
    Ok(())
}

fn handle_size(_fd: &OwnedFd) -> rustix::io::Result<()> {
    println!("SIZE");
    Ok(())
}

fn handle_ping(_fd: &OwnedFd) -> rustix::io::Result<()> {
    println!("PING");
    Ok(())
}

fn handle_flush(_fd: &OwnedFd) -> rustix::io::Result<()> {
    println!("FLUSH");
    Ok(())
}
