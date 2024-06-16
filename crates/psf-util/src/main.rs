use std::fs;

use psf_util::{psf2txt, txt2psf, ungzip, PsfFont};

const DEFAULT_FONT: &str = "/usr/share/kbd/consolefonts/default8x16.psfu.gz";

fn main() {
    write_psf();
}

#[allow(unused)]
fn read_psf_totxt() {
    let bytes = ungzip(&fs::read(DEFAULT_FONT).unwrap());
    let psf = PsfFont::try_from(bytes.as_slice()).unwrap();
    psf.print_table();
    psf2txt(
        &mut fs::File::create("default8x16.psftxt").unwrap(),
        DEFAULT_FONT,
        &psf,
    )
    .unwrap();
}

fn write_psf() {
    let psfscript = fs::read_to_string("contrib/default8x16.psftxt").unwrap();
    let mut lines = psfscript.lines();
    let Some(("load", loadfile)) = lines.next().unwrap().split_once(' ') else {
        panic!("expected \"load <filename>\"");
    };

    let bytes = ungzip(&fs::read(loadfile).unwrap());
    let mut psf = PsfFont::try_from(bytes.as_slice()).unwrap();
    txt2psf(lines, &mut psf).unwrap();
    psf.double_size();

    psf.write_to(&mut fs::File::create("patched16x32.psfu").unwrap())
        .unwrap();
}
