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

    let mut cmd_output = String::from_utf8(cmd_output).unwrap();
    let half_git = cmd_output.len() / 2;
    let half_git = cmd_output[half_git..].find(' ').unwrap() + half_git;
    cmd_output.replace_range(half_git..=half_git, "\n");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("git.txt");
    // note that we have to remove the trailing newline so that we only print two lines, otherwise
    // we start scrolling and other things move up a line.
    fs::write(
        dest_path,
        format!("{{DIM}}{}", cmd_output.trim_matches('\n')),
    )
    .unwrap();

    println!("cargo::rerun-if-changed=.git/HEAD");
}
