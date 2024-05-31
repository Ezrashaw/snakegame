use psf_util::read_psf;

fn main() {
    let psf = read_psf(include_bytes!("../../default8x16.psfu"));
    psf.print_table();
}
