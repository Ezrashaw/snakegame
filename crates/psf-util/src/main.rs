use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

use psf_util::read_psf;

fn main() {
    let bytes = ungzip(&fs::read("/usr/share/kbd/consolefonts/default8x16.psfu.gz").unwrap());
    let psf = read_psf(&bytes);
    psf.print_table();
}

fn ungzip(bytes: &[u8]) -> Vec<u8> {
    let mut cmd = Command::new("gzip")
        .arg("-d")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    cmd.stdin.as_mut().unwrap().write_all(bytes).unwrap();
    cmd.wait_with_output().unwrap().stdout
}
