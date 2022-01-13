extern crate anyhow;
extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_term;

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, load_yaml, App};
use slog::{Drain, Logger, PushFnValue, PushFnValueSerializer, Record};
use std::net::SocketAddr;
use std::process::exit;

fn main() -> Result<()> {
    let yaml = load_yaml!("server-cli.yml");
    let m = App::from_yaml(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .get_matches();

    let addr = match m.value_of("addr") {
        Some(val) => val.to_string(),
        None => "127.0.0.1:4000".to_string(),
    };
    let socket_addr: SocketAddr = match addr.parse() {
        Ok(val) => val,
        Err(_e) => {
            println!("The address {} is invalid", &addr);
            exit(-1);
        }
    };
    let engine = match m.value_of("engine") {
        Some(val) => val.to_string(),
        None => "kvs".to_string(),
    };

    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let logger = Logger::root(
        slog_term::FullFormat::new(plain).build().fuse(),
        o!("src" => PushFnValue(|r: &Record, ser: PushFnValueSerializer| {
            ser.emit(format_args!("{}:{}", r.file(), r.line()))
        })),
    );
    info!(logger, "Server version: {}", crate_version!());
    info!(logger, "Run with {} engine", engine);
    info!(logger, "Listening on {}", addr);

    Ok(())
}
