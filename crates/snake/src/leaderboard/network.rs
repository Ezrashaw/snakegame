use std::{
    fs, io,
    net::{Ipv4Addr, TcpStream},
    time::Duration,
};

use oca_io::network::{self as oca_network, LeaderboardEntries};

use super::Leaderboard;

impl Leaderboard {
    pub(super) fn read_leaderboard(&mut self, block: bool) -> Option<LeaderboardEntries> {
        if !block && !oca_io::poll_read_fd(&self.conn, Some(Duration::ZERO)) {
            return None;
        }

        read_leaderboard(&mut self.conn).ok()
    }

    pub fn send_game(&mut self, name: [u8; 3], score: u8) -> io::Result<()> {
        let mut packet = [0u8; 4];
        packet[0..3].copy_from_slice(&name);
        packet[3] = score;
        oca_network::write_packet(&mut self.conn, 0x1, &packet)
    }
}

pub(super) fn try_tcp() -> io::Result<(LeaderboardEntries, TcpStream)> {
    let mut conn = TcpStream::connect((Ipv4Addr::LOCALHOST, 1234))?;

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
