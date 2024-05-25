use std::{
    io::{self, Write},
    net::{Ipv4Addr, TcpListener},
    thread::sleep,
    time::Duration,
};

fn main() -> io::Result<()> {
    let server = TcpListener::bind((Ipv4Addr::LOCALHOST, 1234))?;
    for incoming in server.incoming() {
        let mut stream = incoming?;
        println!("got incoming {:?}", stream.peer_addr());

        writeln!(stream, "-1-\x00")?;
        writeln!(stream, "-2-\x00")?;
        writeln!(stream, "-3-\x00")?;
        writeln!(stream, "-4-\x00")?;
        writeln!(stream, "-5-\x00")?;
        writeln!(stream, "-6-\x00")?;
        writeln!(stream, "-7-\x00")?;
        writeln!(stream, "-8-\x00")?;
        writeln!(stream, "-9-\x00")?;
        writeln!(stream, "-0-\x00")?;

        sleep(Duration::from_secs(10));

        writeln!(stream, "-1-\x28")?;
        writeln!(stream, "-2-\x23")?;
        writeln!(stream, "-3-\x22")?;
        writeln!(stream, "-4-\x21")?;
        writeln!(stream, "-5-\x19")?;
        writeln!(stream, "-6-\x17")?;
        writeln!(stream, "-7-\x11")?;
        writeln!(stream, "-8-\x11")?;
        writeln!(stream, "-9-\x08")?;
        writeln!(stream, "-0-\x06")?;
        sleep(Duration::from_secs(30));
    }

    Ok(())
}
