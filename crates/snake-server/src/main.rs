use std::{
    io, iter,
    net::{Ipv4Addr, TcpListener, TcpStream},
};

use oca_io::network::{read_packet, write_packet, LeaderboardEntry};

fn main() -> io::Result<()> {
    let server = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 1234))?;
    let mut leaderboard = Vec::new();

    let (stream, _addr) = server.accept()?;
    let mut client = GameClient::new(stream)?;
    client.send_leaderboard(&leaderboard)?;

    loop {
        let (id, packet) = read_packet(&mut client.stream)?;
        assert_eq!(id, 0x1);

        let game = (packet[0..3].try_into().unwrap(), packet[3]);
        let pos = leaderboard
            .binary_search_by(|(_, score)| game.1.cmp(score))
            .map(|e| e + 1)
            .unwrap_or_else(|e| e);
        leaderboard.insert(pos, game);

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
        for (idx, entry) in leaderboard
            .iter()
            .chain(iter::repeat(&(*b"---", 0)))
            .take(10)
            .enumerate()
        {
            let idx = idx * 4;
            lb_packet[idx..(idx + 3)].copy_from_slice(&entry.0);
            lb_packet[idx + 3] = entry.1;
        }

        write_packet(&mut self.stream, 0x0, &lb_packet)
    }
}
