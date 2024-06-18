use crate::{file::File, Result};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct LeaderboardEntry(pub [u8; 3], pub u8);

pub type LeaderboardEntries = [LeaderboardEntry; 10];

pub fn read_packet(r: &mut File) -> Result<(u8, Vec<u8>)> {
    let mut header = [0u8; 3];
    r.read(&mut header)?;

    let packet_id = header[0];
    let packet_size = u16::from_be_bytes(header[1..=2].try_into().unwrap());

    let mut packet = vec![0u8; packet_size.into()];
    r.read(&mut packet)?;

    Ok((packet_id, packet))
}

pub fn write_packet(w: &mut File, id: u8, packet: &[u8]) -> Result<()> {
    let len = (packet.len() as u16).to_be_bytes();
    w.write(&[id, len[0], len[1]])?;
    w.write(packet)?;
    Ok(())
}
