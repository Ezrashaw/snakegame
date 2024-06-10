use std::io;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct LeaderboardEntry(pub [u8; 3], pub u8);

pub type LeaderboardEntries = [LeaderboardEntry; 10];

pub fn read_packet(r: &mut impl io::Read) -> io::Result<(u8, Vec<u8>)> {
    let mut packet_id = [0u8; 1];
    r.read_exact(&mut packet_id)?;
    let packet_id = u8::from_be_bytes(packet_id);

    let mut packet_size = [0u8; 2];
    r.read_exact(&mut packet_size)?;
    let packet_size = u16::from_be_bytes(packet_size);

    let mut packet = vec![0u8; packet_size.into()];
    r.read_exact(&mut packet)?;

    Ok((packet_id, packet))
}

pub fn write_packet(w: &mut impl io::Write, packet_id: u8, packet: &[u8]) -> io::Result<()> {
    w.write_all(&packet_id.to_be_bytes())?;
    w.write_all(&(packet.len() as u16).to_be_bytes())?;
    w.write_all(packet)
}
