use crate::{Result, file::File};

use super::svec::StaticVec;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct LeaderboardEntry(pub [u8; 3], pub u8);

pub type LeaderboardEntries = [LeaderboardEntry; 10];

pub const MAX_PACKET_SIZE: usize = 1024;

pub fn read_packet(r: &mut File) -> Result<(u8, StaticVec<u8, MAX_PACKET_SIZE>)> {
    let mut header = [0u8; 3];
    let n = r.read(&mut header)?;
    assert!(n == 3);

    let id = header[0];
    let len = u16::from_be_bytes(header[1..=2].try_into().unwrap());

    let mut packet = StaticVec::new();
    let n = r.read_uninit(&mut packet, len.into())?;
    assert!(n == len.into());

    Ok((id, packet))
}

pub fn write_packet(w: &mut File, id: u8, packet: &[u8]) -> Result<()> {
    let len = TryInto::<u16>::try_into(packet.len())?.to_be_bytes();
    w.write(&[id, len[0], len[1]])?;
    w.write(packet)?;
    Ok(())
}
