use std::{
    io::{self, Write},
    net::{Ipv4Addr, TcpListener},
};

fn main() -> io::Result<()> {
    let server = TcpListener::bind((Ipv4Addr::LOCALHOST, 1234))?;
    for incoming in server.incoming() {
        let mut stream = incoming?;
        println!("got incoming {:?}", stream.peer_addr());

        writeln!(stream, "-bobby1-\x28")?;
        writeln!(stream, "-bobby2-\x23")?;
        writeln!(stream, "-bobby3-\x22")?;
        writeln!(stream, "-bobby4-\x21")?;
        writeln!(stream, "-bobby5-\x19")?;
        writeln!(stream, "-bobby6-\x17")?;
        writeln!(stream, "-bobby7-\x11")?;
        writeln!(stream, "-bobby8-\x11")?;
        writeln!(stream, "-bobby9-\x08")?;
    }

    Ok(())
}
