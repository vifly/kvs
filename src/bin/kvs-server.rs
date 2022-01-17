extern crate clap;
#[macro_use(o)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;

use std::net::SocketAddr;
use std::process::exit;

use clap::{App, crate_authors, crate_description, crate_version, load_yaml};
use slog::{Drain, PushFnValue, PushFnValueSerializer, Record};

use kvs::KvsServer;

fn main() {
    let yaml = load_yaml!("server-cli.yml");
    let m = App::from_yaml(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .get_matches();

    let addr = m.value_of("addr").unwrap_or("127.0.0.1:4000").to_string();
    let socket_addr: SocketAddr = match addr.parse() {
        Ok(val) => val,
        Err(_e) => {
            println!("The address {} is invalid", &addr);
            exit(-1);
        }
    };
    let engine = m.value_of("engine").unwrap_or("kvs").to_string();

    let plain = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let logger = slog::Logger::root(
        slog_term::FullFormat::new(plain).build().fuse(),
        o!("src" => PushFnValue(|r: &Record, ser: PushFnValueSerializer| {
            ser.emit(format_args!("{}:{}", r.file(), r.line()))
        })),
    );
    let _guard = slog_scope::set_global_logger(logger);

    info!("Server version: {}", crate_version!());
    info!("Run with {} engine", engine);
    info!("Listening on {}", addr);

    let mut server = KvsServer::new(socket_addr, engine);
    server.handle_connection();
}
