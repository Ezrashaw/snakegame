use std::{
    io, iter,
    net::{Ipv4Addr, TcpListener, TcpStream},
    os::fd::AsRawFd,
};

use oca_io::{
    network::{read_packet, write_packet, LeaderboardEntry},
    PollFd,
};

fn main() -> io::Result<()> {
    let mut leaderboard = Vec::new();

    let server = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 1111))?;
    let mut clients = Vec::new();
    let mut poll_fds = vec![PollFd::new_read(&server)];

    loop {
        let number_read = oca_io::poll(&mut poll_fds, None);
        assert!(number_read == 1);

        let poll_fd = poll_fds
            .iter()
            .find(|fd| fd.is_read() || fd.is_sock_closed())
            .unwrap();

        if poll_fd.fd() == server.as_raw_fd() {
            assert!(!poll_fd.is_sock_closed());

            let (stream, _addr) = server.accept()?;
            let mut client = GameClient::new(stream)?;
            client.send_leaderboard(&leaderboard)?;
            poll_fds.push(PollFd::new_socket(&client.stream));
            clients.push(client);
        } else {
            let client = clients
                .iter_mut()
                .find(|cl| cl.stream.as_raw_fd() == poll_fd.fd())
                .unwrap();

            if poll_fd.is_sock_closed() {
                println!("{:?}: DISCONNECT", client.stream.peer_addr()?);
                let fd = client.stream.as_raw_fd();
                let idx = clients
                    .iter()
                    .position(|c| c.stream.as_raw_fd() == fd)
                    .unwrap();
                clients.remove(idx);
                poll_fds.remove(idx + 1);
                continue;
            }

            client.handle_packet(&mut leaderboard)?;

            for i in 0..clients.len() {
                let client = &mut clients[i];
                client.send_leaderboard(&leaderboard)?;
            }
        }
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
        println!("{:?}: CONNECT {}", stream.peer_addr()?, hostname);

        Ok(Self { stream, hostname })
    }

    pub fn handle_packet(&mut self, leaderboard: &mut Vec<LeaderboardEntry>) -> io::Result<()> {
        let (id, packet) = read_packet(&mut self.stream).unwrap();
        assert_eq!(id, 0x1);

        let game = (packet[0..3].try_into().unwrap(), packet[3]);
        let pos = leaderboard
            .binary_search_by(|(_, score)| game.1.cmp(score))
            .map(|e| e + 1)
            .unwrap_or_else(|e| e);

        leaderboard.insert(pos, game);

        let name = std::str::from_utf8(&game.0).unwrap();
        println!("{:?}: GAME {} {}", self.stream.peer_addr()?, name, game.1);
        Ok(())
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
