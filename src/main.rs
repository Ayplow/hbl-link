use clap::Clap;
use hbl_link::*;
use std::path::PathBuf;

#[derive(Clap, Debug)]
struct Opts {
    /// Path to the .3dsx file to upload
    file: PathBuf,
    /// Hostname or IPv4 address of 3DS
    #[clap(short, long)]
    address: Option<std::net::Ipv4Addr>,
    #[clap(short, long, default_value = "10")]
    /// Number of times to ping before giving up
    retries: usize,
}
fn main() -> std::io::Result<()> {
    let opts = Opts::parse();

    let addr = find_3ds(opts.retries)?;

    let file = std::fs::File::open(&opts.file)?;

    send_3dsx_file(addr, opts.file.file_name().unwrap().to_str().unwrap(), file)?;

    Ok(())
}
