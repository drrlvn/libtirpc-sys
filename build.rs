use lazy_static::lazy_static;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const LIBTIRPC_VERSION: &str = "1.1.4";
lazy_static! {
    static ref OUT_DIR: PathBuf = PathBuf::from(env::var("OUT_DIR").unwrap());
    static ref LIBTIRPC_DIR: PathBuf = OUT_DIR.join(format!("libtirpc-{}", LIBTIRPC_VERSION));
    static ref LIBTIRPC_OUTPUT_DIR: PathBuf = OUT_DIR.join("libtirpc");
}

fn run<P: AsRef<Path>>(mut cmd: Command, path: P) {
    let dir = OUT_DIR.join(path.as_ref());
    println!("Running {:?} in {:?}", cmd, dir);
    cmd.current_dir(dir).status().unwrap();
}

fn download_and_extract() {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(format!("curl -L https://downloads.sourceforge.net/sourceforge/libtirpc/libtirpc-{}.tar.bz2 | tar xjf -", LIBTIRPC_VERSION));
    run(cmd, "");
}

fn configure() {
    let mut cmd = Command::new("./configure");
    cmd.arg(format!("--prefix={}", LIBTIRPC_OUTPUT_DIR.display()))
        .arg("--disable-gssapi");
    run(cmd, &*LIBTIRPC_DIR);
}

fn make() {
    let cmd = Command::new("make");
    run(cmd, &*LIBTIRPC_DIR);
}

fn install() {
    let mut cmd = Command::new("make");
    cmd.arg("install");
    run(cmd, &*LIBTIRPC_DIR);
}

fn main() {
    if !LIBTIRPC_DIR.exists() {
        download_and_extract();
    }

    if !LIBTIRPC_DIR.join("Makefile").exists() {
        configure();
    }
    make();
    install();

    println!(
        "cargo:rustc-link-search=native={}/lib",
        LIBTIRPC_OUTPUT_DIR.display()
    );
    println!("cargo:rustc-link-lib=static=tirpc");

    bindgen::Builder::default()
        .header(LIBTIRPC_DIR.join("tirpc/rpc/rpc.h").display().to_string())
        .blacklist_type("rpcblist")
        .clang_arg(format!("-I{}/tirpc", LIBTIRPC_DIR.display()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(OUT_DIR.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
