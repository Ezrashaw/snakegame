use std::{
    io::{self, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
};

use oca_io::network::{read_packet, write_packet, LeaderboardEntry};

fn main() -> io::Result<()> {
    let server = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 1234))?;
    let mut clients = Vec::new();
    let leaderboard = vec![
        (*b"AAA", 100),
        (*b"BBB", 90),
        (*b"CCC", 80),
        (*b"DDD", 70),
        (*b"EEE", 60),
        (*b"FFF", 50),
        (*b"GGG", 40),
        (*b"HHH", 30),
        (*b"III", 20),
        (*b"JJJ", 10),
    ];

    for stream in server.incoming() {
        let mut client = GameClient::new(stream?)?;
        client.send_leaderboard(&leaderboard)?;
        clients.push(client);
    }

    Ok(())
}

pub struct GameClient {
    stream: TcpStream,
    #[allow(unused)]
    hostname: String,
}

impl GameClient {
    pub fn new(mut stream: TcpStream) -> io::Result<Self> {
        let (connect_id, connect) = read_packet(&mut stream)?;
        assert_eq!(connect_id, 0x0);

        let hostname = String::from_utf8(connect).unwrap();
        println!("got incoming {:?}: {}", stream.peer_addr(), hostname);

        Ok(Self { stream, hostname })
    }

    pub fn send_leaderboard(&mut self, leaderboard: &[LeaderboardEntry]) -> io::Result<()> {
        let mut lb_packet = Vec::new();
        for entry in &leaderboard[0..10] {
            lb_packet.write_all(&entry.0)?;
            lb_packet.write_all(&entry.1.to_be_bytes())?;
        }

        write_packet(&mut self.stream, 0x0, &lb_packet)
    }
}
