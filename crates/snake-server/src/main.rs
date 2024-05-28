use std::{
    io,
    net::{Ipv4Addr, TcpListener, TcpStream},
};

use oca_io::network::{read_packet, write_packet, LeaderboardEntry};

fn main() -> io::Result<()> {
    let server = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 1234))?;
    let mut leaderboard = vec![
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

    let (stream, _addr) = server.accept()?;
    let mut client = GameClient::new(stream)?;
    client.send_leaderboard(&leaderboard)?;

    loop {
        let (id, packet) = read_packet(&mut client.stream)?;
        assert_eq!(id, 0x1);

        leaderboard[0].0 = packet[0..3].try_into().unwrap();
        leaderboard[0].1 = packet[3];

        client.send_leaderboard(&leaderboard)?;
    }
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
        let mut lb_packet = [0u8; 40];
        for (idx, entry) in leaderboard[0..10].iter().enumerate() {
            let idx = idx * 4;
            lb_packet[idx..(idx + 3)].copy_from_slice(&entry.0);
            lb_packet[idx + 3] = entry.1;
        }

        write_packet(&mut self.stream, 0x0, &lb_packet)
    }
}
