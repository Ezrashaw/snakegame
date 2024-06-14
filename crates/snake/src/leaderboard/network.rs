use std::{
    fs, io,
    net::{SocketAddr, TcpStream},
    str::FromStr,
    thread,
    time::Duration,
};

use oca_io::network::{self as oca_network, LeaderboardEntries};

use super::Leaderboard;

impl Leaderboard {
    pub(super) fn read_leaderboard(&mut self, block: bool) -> Option<LeaderboardEntries> {
        let conn = match self.conn.as_mut() {
            Ok(conn) => conn,
            Err(thread) => {
                if thread.as_ref().unwrap().is_finished() {
                    if let Ok((entries, conn)) = thread.take().unwrap().join().unwrap() {
                        println!("\x1B[H ");
                        self.conn = Ok(conn);
                        return Some(entries);
                    }

                    let addr = self.addr.clone();
                    self.conn = Err(Some(thread::spawn(move || connect_tcp(&addr))));
                }
                return None;
            }
        };

        if !block && !oca_io::poll::poll_read_fd(conn, Some(Duration::ZERO)) {
            return None;
        }

        read_leaderboard(conn)
            .inspect_err(|_| {
                println!("\x1B[H\x1B[1;31m▀\x1B[0m");
                let addr = self.addr.clone();
                self.conn = Err(Some(thread::spawn(move || connect_tcp(&addr))));
            })
            .ok()
    }

    pub const fn has_conn(&self) -> bool {
        self.conn.is_ok()
    }

    pub fn send_game(&mut self, name: [u8; 3], score: u8) -> io::Result<()> {
        let mut packet = [0u8; 4];
        packet[0..3].copy_from_slice(&name);
        packet[3] = score;
        oca_network::write_packet(self.conn.as_mut().unwrap(), 0x1, &packet)
    }
}

pub(super) fn connect_tcp(addr: &str) -> io::Result<(LeaderboardEntries, TcpStream)> {
    let mut conn = TcpStream::connect_timeout(
        &SocketAddr::from_str(addr).map_err(io::Error::other)?,
        Duration::from_secs(10),
    )?;

    let hostname = fs::read_to_string("/proc/sys/kernel/hostname")?;
    oca_network::write_packet(&mut conn, 0x0, hostname.trim().as_bytes())?;

    let lb = read_leaderboard(&mut conn)?;

    Ok((lb, conn))
}

fn read_leaderboard(stream: &mut TcpStream) -> io::Result<LeaderboardEntries> {
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
