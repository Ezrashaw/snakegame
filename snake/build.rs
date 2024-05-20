use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let cmd_output = Command::new("git")
        .args(["show", "HEAD", "--format=reference", "--no-patch"])
        .output()
        .unwrap()
        .stdout;

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("git.txt");
    fs::write(dest_path, cmd_output).unwrap();

    println!("cargo::rerun-if-changed=.git/HEAD");
}
