use hashbrown::HashMap;
use rustix::fd::OwnedFd;
use rustix::io::{read, write};

pub const MAX_KEY_LEN: usize = 16;
pub const MAX_VALUE_LEN: usize = 32;

pub const MAX_CONNECTIONS: usize = 10_000;
pub const MAX_ENTRIES: usize = 1_000_000;

pub type KVStore =
    HashMap<[u8; MAX_KEY_LEN], [u8; MAX_VALUE_LEN]>;

pub const MAGIC_BYTES: [u8; 2] = [b'v', b'k'];

pub const SET_KEY:    [u8; 1] = [0x01];
pub const GET_KEY:    [u8; 1] = [0x02];
pub const DELETE_KEY: [u8; 1] = [0x03];
pub const SIZE:       [u8; 1] = [0x04];
pub const PING:       [u8; 1] = [0x05];
pub const FLUSH:      [u8; 1] = [0x06];

pub const OK:                [u8; 1] = [0x69];
pub const KEY_NOT_FOUND:     [u8; 1] = [0x67];
pub const MAX_LIMIT_REACHED: [u8; 1] = [0x68];
pub const OK_KEY_UPDATED:    [u8; 1] = [0x65];
pub const INVALID_OPCODE:    [u8; 1] = [0x64];

pub fn readn(
    fd: &OwnedFd,
    buf: &mut [u8],
) -> rustix::io::Result<usize> {
    let mut total = 0;

    while total < buf.len() {
        match read(fd, &mut buf[total..])? {
            0 => break,
            n => total += n,
        }
    }

    Ok(total)
}

pub fn writen(
    fd: &OwnedFd,
    buf: &[u8],
) -> rustix::io::Result<()> {
    let mut total = 0;

    while total < buf.len() {
        let n = write(fd, &buf[total..])?;
        total += n;
    }

    Ok(())
}

pub fn parser(
    fd: &OwnedFd,
    store: &mut KVStore,
) -> rustix::io::Result<()> {
    let mut magic = [0u8; 2];

    readn(fd, &mut magic)?;

    if magic != MAGIC_BYTES {
        return Ok(());
    }

    let mut opcode = [0u8; 1];

    readn(fd, &mut opcode)?;

    match opcode {
        SET_KEY    => handle_set(fd, store)?,
        GET_KEY    => handle_get(fd, store)?,
        DELETE_KEY => handle_delete(fd, store)?,
        SIZE       => handle_size(fd, store)?,
        PING       => handle_ping(fd)?,
        FLUSH      => handle_flush(fd, store)?,
        _          => writen(fd, &INVALID_OPCODE)?,
    }

    Ok(())
}

fn handle_set(
    fd: &OwnedFd,
    store: &mut KVStore,
) -> rustix::io::Result<()> {
    let mut key = [0u8; MAX_KEY_LEN];
    let mut value = [0u8; MAX_VALUE_LEN];

    readn(fd, &mut key)?;
    readn(fd, &mut value)?;

    let updating = store.contains_key(&key);

    if !updating && store.len() >= MAX_ENTRIES {
        writen(fd, &MAX_LIMIT_REACHED)?;
        return Ok(());
    }

    let existed = store.insert(key, value).is_some();

    if existed {
        writen(fd, &OK_KEY_UPDATED)?;
    } else {
        writen(fd, &OK)?;
    }

    Ok(())
}

fn handle_get(
    fd: &OwnedFd,
    store: &KVStore,
) -> rustix::io::Result<()> {
    let mut key = [0u8; MAX_KEY_LEN];

    readn(fd, &mut key)?;

    match store.get(&key) {
        Some(value) => {
            writen(fd, &OK)?;
            writen(fd, value)?;
        }
        None => {
            writen(fd, &KEY_NOT_FOUND)?;
        }
    }

    Ok(())
}

fn handle_delete(
    fd: &OwnedFd,
    store: &mut KVStore,
) -> rustix::io::Result<()> {
    let mut key = [0u8; MAX_KEY_LEN];

    readn(fd, &mut key)?;

    if store.remove(&key).is_some() {
        writen(fd, &OK)?;
    } else {
        writen(fd, &KEY_NOT_FOUND)?;
    }

    Ok(())
}

fn handle_size(
    fd: &OwnedFd,
    store: &KVStore,
) -> rustix::io::Result<()> {
    let size = store.len() as u64;

    writen(fd, &OK)?;
    writen(fd, &size.to_le_bytes())?;

    Ok(())
}

fn handle_ping(
    fd: &OwnedFd,
) -> rustix::io::Result<()> {
    writen(fd, &OK)?;
    Ok(())
}

fn handle_flush(
    fd: &OwnedFd,
    store: &mut KVStore,
) -> rustix::io::Result<()> {
    store.clear();

    writen(fd, &OK)?;

    Ok(())
}
