use std::fs;

use psf_util::{psf2txt, read_psf, txt2psf, ungzip};

const DEFAULT_FONT: &str = "/usr/share/kbd/consolefonts/default8x16.psfu.gz";

fn main() {
    write_psf();
}

#[allow(unused)]
fn read_psf_totxt() {
    let bytes = ungzip(&fs::read(DEFAULT_FONT).unwrap());
    let psf = read_psf(&bytes);
    psf.print_table();
    psf2txt(
        &mut fs::File::create("default8x16.psftxt").unwrap(),
        DEFAULT_FONT,
        &psf,
    )
    .unwrap();
}

fn write_psf() {
    let psfscript = fs::read_to_string("../../contrib/default8x16.psftxt").unwrap();
    let mut lines = psfscript.lines();
    let Some(("load", loadfile)) = lines.next().unwrap().split_once(' ') else {
        panic!("expected \"load <filename>\"");
    };

    let bytes = ungzip(&fs::read(loadfile).unwrap());
    let mut psf = read_psf(&bytes);
    txt2psf(lines, &mut psf).unwrap();

    psf.write_psf(&mut fs::File::create("patched8x16.psfu").unwrap())
        .unwrap();
}
