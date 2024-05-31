use std::fs;

use psf_util::{read_psf, ungzip};

fn main() {
    let bytes = ungzip(&fs::read("/usr/share/kbd/consolefonts/default8x16.psfu.gz").unwrap());
    let psf = read_psf(&bytes);
    psf.print_table();
}
