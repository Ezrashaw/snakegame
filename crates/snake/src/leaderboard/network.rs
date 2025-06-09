use core::net::SocketAddrV4;
use std::fs;

use oca_io::{
    Result,
    network::{self as oca_network, LeaderboardEntries},
    socket::Socket,
};

use super::Leaderboard;

impl Leaderboard {
    pub(super) fn read_leaderboard(&mut self, block: bool) -> Option<LeaderboardEntries> {
        if !self.sock_is_conn {
            match self.sock.sock_finish_conn() {
                Ok(true) => {
                    self.sock_is_conn = true;
                    let hostname = fs::read_to_string("/proc/sys/kernel/hostname").unwrap();
                    oca_network::write_packet(&mut self.sock, 0x0, hostname.trim().as_bytes())
                        .unwrap();

                    println!("\x1B[H ");
                }
                Ok(false) => {
                    return None;
                }
                Err(_) => {
                    self.sock = Socket::connect(self.addr, false).unwrap();
                    self.sock_is_conn = self.sock.is_conn();

                    println!("\x1B[H\x1B[91;1mN\x1B[0m");
                }
            }
        }

        if !block {
            match self.sock.poll() {
                Ok(Some(true)) => (),           // got data
                Ok(Some(false)) => return None, // no data; no error
                Err(_) | Ok(None) => {
                    self.sock = Socket::connect(self.addr, false).unwrap();
                    self.sock_is_conn = self.sock.is_conn();

                    println!("\x1B[H\x1B[91;1mN\x1B[0m");
                    return None;
                }
            }
        }

        read_leaderboard(&mut self.sock)
            .inspect_err(|_| {
                self.sock = Socket::connect(self.addr, false).unwrap();
                self.sock_is_conn = self.sock.is_conn();

                if !self.sock_is_conn {
                    println!("\x1B[H\x1B[91;1mN");
                }
            })
            .ok()
    }

    pub const fn has_conn(&self) -> bool {
        self.sock_is_conn
    }

    pub fn send_game(&mut self, name: [u8; 3], score: u8) -> Result<()> {
        let mut packet = [0u8; 4];
        packet[0..3].copy_from_slice(&name);
        packet[3] = score;
        oca_network::write_packet(&mut self.sock, 0x1, &packet)
    }
}

pub(super) fn connect_tcp(addr: SocketAddrV4) -> Result<(LeaderboardEntries, Socket)> {
    let mut conn = Socket::connect(addr, true)?;

    let hostname = fs::read_to_string("/proc/sys/kernel/hostname").unwrap();
    oca_network::write_packet(&mut conn, 0x0, hostname.trim().as_bytes())?;

    let lb = read_leaderboard(&mut conn)?;

    Ok((lb, conn))
}

fn read_leaderboard(stream: &mut Socket) -> Result<LeaderboardEntries> {
    let (packet_id, packet) = oca_network::read_packet(stream)?;
    assert_eq!(packet_id, 0x0);
    assert_eq!(packet.len(), 40);

    let mut entries = LeaderboardEntries::default();
    for (idx, entry) in packet.chunks(4).enumerate() {
        entries[idx].0 = entry[0..3].try_into().unwrap();
        entries[idx].1 = entry[3];
    }

    Ok(entries)
}
