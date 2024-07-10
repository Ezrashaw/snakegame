#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use core::slice;
use std::{
    env, fs, iter,
    mem::ManuallyDrop,
    net::{Ipv4Addr, TcpListener, TcpStream},
    os::fd::AsRawFd,
};

use oca_io::{
    file::File,
    network::{read_packet, write_packet, LeaderboardEntry},
    poll::PollFd,
    Result,
};

fn main() -> Result<()> {
    let mut leaderboard = fs::read("games")
        .ok()
        .map(|lb| {
            let mut lb = ManuallyDrop::new(lb);
            assert!(lb.len() % 4 == 0);
            unsafe { Vec::from_raw_parts(lb.as_mut_ptr().cast(), lb.len() / 4, lb.capacity() / 4) }
        })
        .unwrap_or_default();

    let server = TcpListener::bind((
        Ipv4Addr::UNSPECIFIED,
        env::var("SNAKEPORT").unwrap().parse().unwrap(),
    ))
    .unwrap();
    let mut clients = Vec::new();
    let mut poll_fds = vec![PollFd::new(server.as_raw_fd(), PollFd::IN | PollFd::RDHUP)];

    loop {
        let number_read = oca_io::poll::poll(&mut poll_fds, None)?;
        assert!(number_read == 1);

        let poll_fd = poll_fds
            .iter()
            .find(|fd| fd.has_read() || fd.has_socket_close())
            .unwrap();

        if poll_fd.fd() == server.as_raw_fd() {
            assert!(poll_fd.is_read());

            let (stream, _addr) = server.accept().unwrap();
            let mut client = GameClient::new(stream)?;

            client.send_leaderboard(&leaderboard)?;
            poll_fds.push(PollFd::new(
                client.stream.as_raw_fd(),
                PollFd::IN | PollFd::RDHUP,
            ));
            clients.push(client);
        } else {
            let client = clients
                .iter_mut()
                .find(|cl| cl.stream.as_raw_fd() == poll_fd.fd())
                .unwrap();

            if poll_fd.has_socket_close() {
                println!("{}: DISCONNECT", client.hostname);
                let fd = client.stream.as_raw_fd();
                let idx = clients
                    .iter()
                    .position(|c| c.stream.as_raw_fd() == fd)
                    .unwrap();
                clients.remove(idx);
                poll_fds.remove(idx + 1);
                continue;
            }

            assert!(poll_fd.is_read());
            client.handle_packet(&mut leaderboard).unwrap();

            let bytes = unsafe {
                slice::from_raw_parts(leaderboard.as_ptr().cast(), leaderboard.len() * 4)
            };
            fs::write("games", bytes).unwrap();

            for i in 0..clients.len() {
                if let Err(_err) = clients[i].send_leaderboard(&leaderboard) {
                    println!("{}: DISCONNECT (failed packet write)", clients[i].hostname);
                    clients.remove(i);
                    poll_fds.remove(i + 1);
                }
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
    pub fn new(stream: TcpStream) -> Result<Self> {
        let (connect_id, connect) =
            read_packet(&mut oca_io::file::File::from_fd(stream.as_raw_fd()))?;
        assert_eq!(connect_id, 0x0);

        let hostname = String::from_utf8(connect.to_vec()).unwrap();
        println!("{}: CONNECT {:?}", hostname, stream.peer_addr().unwrap());

        Ok(Self { stream, hostname })
    }

    pub fn handle_packet(&mut self, leaderboard: &mut Vec<LeaderboardEntry>) -> Result<()> {
        let (id, packet) = read_packet(&mut oca_io::file::File::from_fd(self.stream.as_raw_fd()))?;
        assert_eq!(id, 0x1);

        let game = LeaderboardEntry(packet[0..3].try_into().unwrap(), packet[3]);
        let pos = leaderboard
            .binary_search_by(|LeaderboardEntry(_, score)| game.1.cmp(score))
            .map_or_else(|e| e, |e| e + 1);

        leaderboard.insert(pos, game);

        let name = std::str::from_utf8(&game.0).unwrap();
        println!("{}: GAME {} {}", self.hostname, name, game.1);
        Ok(())
    }

    pub fn send_leaderboard(&mut self, leaderboard: &[LeaderboardEntry]) -> Result<()> {
        let mut lb_packet = [0u8; 40];
        for (idx, entry) in leaderboard
            .iter()
            .chain(iter::repeat(&LeaderboardEntry(*b"---", 0)))
            .take(10)
            .enumerate()
        {
            let idx = idx * 4;
            lb_packet[idx..(idx + 3)].copy_from_slice(&entry.0);
            lb_packet[idx + 3] = entry.1;
        }

        write_packet(
            &mut oca_io::file::File::from_fd(self.stream.as_raw_fd()),
            0x0,
            &lb_packet,
        )
    }
}
