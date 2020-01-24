use flate2::Compression;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream, UdpSocket};

pub fn find_3ds(retries: usize) -> io::Result<SocketAddr> {
    let broadcast_sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    broadcast_sock.set_broadcast(true)?;

    let receive_sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 17491))?;
    receive_sock.set_nonblocking(true)?;
    let mut buf = [0; 7];

    for _ in 1..retries {
        std::thread::sleep(std::time::Duration::from_micros(60000));
        broadcast_sock.send_to(b"3dsboot", (Ipv4Addr::BROADCAST, 17491))?;
        match receive_sock.recv_from(&mut buf) {
            Ok((n, src)) if &buf[..n] == b"boot3ds" => return Ok(src),
            Err(e) if e.kind() != io::ErrorKind::WouldBlock => return Err(e),
            _ => (),
        };
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Couldn't find 3ds in the network",
    ))
}

pub fn send_3dsx_file(addr: SocketAddr, filename: &str, file: fs::File) -> io::Result<()> {
    let filelen = file.metadata().unwrap().len();
    let namelen = filename.len();

    let mut sock = TcpStream::connect(addr)?;
    let mut archive = flate2::read::ZlibEncoder::new(file, Compression::best());

    sock.write(&namelen.to_le_bytes()[..4])?;
    sock.write(filename.as_bytes())?;
    sock.write(&filelen.to_le_bytes()[..4])?;

    let mut buf = [0; 4];
    sock.read(&mut buf)?;
    let response = i32::from_be_bytes(buf);
    if response != 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            match response {
                -1 => "Failed to create file",
                -2 => "Insufficient space",
                -3 => "Insufficient memory",
                _ => "",
            },
        ));
    }

    println!("Sending {}, {} bytes", filename, filelen);
    let mut buf = [0; 16 * 1024];
    let mut totalsent = 0;
    let mut blocks = 0;
    loop {
        let n = archive.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sock.write(&n.to_le_bytes()[..4])?;
        sock.write(&buf[..n])?;
        totalsent += n;
        blocks += 1;
    }
    println!(
        "{} sent ({:.2}%), {} blocks",
        totalsent,
        totalsent as f64 * 100.0 / filelen as f64,
        blocks
    );

    Ok(())
}
