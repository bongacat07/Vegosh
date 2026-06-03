use vegosh_db::protocol::*;
use easy_repl::{Repl, CommandStatus, command};
use rustix::net::{AddressFamily, SocketType, connect, ipproto, socket};
use rustix::fd::OwnedFd;
use std::net::{Ipv4Addr, SocketAddrV4};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sockfd = socket(
        AddressFamily::INET,
        SocketType::STREAM,
        Some(ipproto::TCP),
    )?;
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080);
    connect(&sockfd, &addr)?;
    println!("Client started");
    build_run_repl(&sockfd)?;
    Ok(())
}

fn build_run_repl(fd: &OwnedFd) -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::builder()
        .add("SET", command! {
            "SET Key and Value",
            (key: String, value: String) => |k: String, v: String| {
                if k.len() > MAX_KEY_LEN {
                    println!("ERROR: key too long (max {} bytes)", MAX_KEY_LEN);
                    return Ok(CommandStatus::Done);
                }
                if v.len() > MAX_VALUE_LEN {
                    println!("ERROR: value too long (max {} bytes)", MAX_VALUE_LEN);
                    return Ok(CommandStatus::Done);
                }
                set(fd, &k, &v);
                Ok(CommandStatus::Done)
            }
        })
        .add("GET", command! {
            "GET Key",
            (key: String) => |k: String| {
                if k.len() > MAX_KEY_LEN {
                    println!("ERROR: invalid key (max {} bytes)", MAX_KEY_LEN);
                    return Ok(CommandStatus::Done);
                }
                get(fd, &k);
                Ok(CommandStatus::Done)
            }
        })
        .add("DELETE", command! {
            "DELETE Key",
            (key: String) => |k: String| {
                if k.len() > MAX_KEY_LEN {
                    println!("ERROR: invalid key (max {} bytes)", MAX_KEY_LEN);
                    return Ok(CommandStatus::Done);
                }
                delete(fd, &k);
                Ok(CommandStatus::Done)
            }
        })
        .add("PING", command! {
            "PING Server",
            () => || {
                ping(fd);
                Ok(CommandStatus::Done)
            }
        })
        .add("SIZE", command! {
            "SIZE of KVStore",
            () => || {
                size(fd);
                Ok(CommandStatus::Done)
            }
        })
        .add("FLUSH", command! {
            "FLUSH KVStore",
            () => || {
                flush(fd);
                Ok(CommandStatus::Done)
            }
        })
        .build()?;
    repl.run()?;
    Ok(())
}

// --- helpers ---

fn to_key(s: &str) -> [u8; MAX_KEY_LEN] {
    let mut buf = [0u8; MAX_KEY_LEN];
    let bytes = s.as_bytes();
    let len = bytes.len().min(MAX_KEY_LEN);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}

fn to_value(s: &str) -> [u8; MAX_VALUE_LEN] {
    let mut buf = [0u8; MAX_VALUE_LEN];
    let bytes = s.as_bytes();
    let len = bytes.len().min(MAX_VALUE_LEN);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}

fn parse_response(response: &[u8]) {
    match response {
        [0x69] => println!("OK"),
        [0x67] => println!("KEY NOT FOUND"),
        [0x68] => println!("MAX LIMIT REACHED"),
        [0x65] => println!("OK KEY UPDATED"),
        [0x64] => println!("INVALID OPCODE"),
        _      => println!("UNKNOWN RESPONSE: {:?}", response),
    }
}

// --- commands ---

fn set(fd: &OwnedFd, key: &str, value: &str) {
    writen(fd, &MAGIC_BYTES).unwrap();
    writen(fd, &SET_KEY).unwrap();
    writen(fd, &to_key(key)).unwrap();
    writen(fd, &to_value(value)).unwrap();
    let mut response = [0u8; 1];
    readn(fd, &mut response).unwrap();
    parse_response(&response);
}

fn get(fd: &OwnedFd, key: &str) {
    writen(fd, &MAGIC_BYTES).unwrap();
    writen(fd, &GET_KEY).unwrap();
    writen(fd, &to_key(key)).unwrap();
    let mut response = [0u8; 1];
    readn(fd, &mut response).unwrap();
    match response {
        [0x69] => {
            let mut value = [0u8; MAX_VALUE_LEN];
            readn(fd, &mut value).unwrap();
            let s = std::str::from_utf8(&value)
                .unwrap_or("")
                .trim_end_matches('\0');
            println!("VALUE: {}", s);
        }
        _ => parse_response(&response),
    }
}

fn delete(fd: &OwnedFd, key: &str) {
    writen(fd, &MAGIC_BYTES).unwrap();
    writen(fd, &DELETE_KEY).unwrap();
    writen(fd, &to_key(key)).unwrap();
    let mut response = [0u8; 1];
    readn(fd, &mut response).unwrap();
    parse_response(&response);
}

fn ping(fd: &OwnedFd) {
    writen(fd, &MAGIC_BYTES).unwrap();
    writen(fd, &PING).unwrap();
    let mut response = [0u8; 1];
    readn(fd, &mut response).unwrap();
    parse_response(&response);
}

fn size(fd: &OwnedFd) {
    writen(fd, &MAGIC_BYTES).unwrap();
    writen(fd, &SIZE).unwrap();
    let mut response = [0u8; 1];
    readn(fd, &mut response).unwrap();
    parse_response(&response);
    let mut size_bytes = [0u8; 8];
    readn(fd, &mut size_bytes).unwrap();
    let size = u64::from_le_bytes(size_bytes);
    println!("SIZE: {}", size);
}

fn flush(fd: &OwnedFd) {
    writen(fd, &MAGIC_BYTES).unwrap();
    writen(fd, &FLUSH).unwrap();
    let mut response = [0u8; 1];
    readn(fd, &mut response).unwrap();
    parse_response(&response);
}
