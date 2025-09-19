use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| help_and_exit(2));
    match cmd.as_str() {
        // Legacy commands (not documented): install/uninstall
        "install" | "man-install" => { if let Err(e) = man_install() { exit_err(e) } }
        "uninstall" | "man-uninstall" => { if let Err(e) = man_uninstall() { exit_err(e) } }
        _ => help_and_exit(2),
    }
}

fn help_and_exit(code: i32) -> ! {
    eprintln!(
        "xtask usage:\n  cargo man-install      # install man page\n  cargo man-uninstall    # remove man page\n\nBinary install is handled by: cargo install --path .\n\nENV:\n  PREFIX   install prefix (default: /usr/local)\n  DESTDIR  optional staging root (prepended to PREFIX)\n"
    );
    std::process::exit(code);
}

fn man_install() -> io::Result<()> {
    let (_dest_bin, dest_man) = destinations();
    let src_man = project_root().join("man/ykchalresp.1");
    if let Some(dir) = dest_man.parent() { fs::create_dir_all(dir)?; }
    fs::copy(&src_man, &dest_man)?;
    println!("Installed man page:\n  {}", display(&dest_man));
    Ok(())
}

fn man_uninstall() -> io::Result<()> {
    let (_dest_bin, dest_man) = destinations();
    if dest_man.exists() {
        fs::remove_file(&dest_man)?;
        println!("Removed {}", display(&dest_man));
    } else {
        println!("Man page not found: {}", display(&dest_man));
    }
    Ok(())
}

fn run(cmd: &mut Command) -> io::Result<()> {
    let status = cmd.status()?;
    if !status.success() {
        Err(io::Error::new(io::ErrorKind::Other, format!("command failed: {:?}", cmd)))
    } else {
        Ok(())
    }
}

fn project_root() -> PathBuf {
    // Walk up from CARGO_MANIFEST_DIR
    let dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(dir)
}

fn prefix() -> PathBuf {
    let destdir = env::var_os("DESTDIR").map(PathBuf::from).unwrap_or_default();
    let prefix = env::var_os("PREFIX").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/usr/local"));
    if destdir.as_os_str().is_empty() { prefix } else { destdir.join(prefix.strip_prefix("/").unwrap_or(&prefix)) }
}

fn destinations() -> (PathBuf, PathBuf) {
    let pfx = prefix();
    let bin = pfx.join("bin/ykchalresp");
    let man = pfx.join("share/man/man1/ykchalresp.1");
    (bin, man)
}

fn display(p: &Path) -> String { p.display().to_string() }

fn exit_err(e: io::Error) -> ! {
    eprintln!("error: {}", e);
    std::process::exit(1);
}
