#[macro_use(o)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;
extern crate strum;

use std::net::SocketAddr;
use std::process::exit;

use argh::FromArgs;
use slog::{Drain, PushFnValue, PushFnValueSerializer, Record};


use kvs::KvsServer;

#[derive(Debug, Eq, PartialEq, strum_macros::Display, strum_macros::EnumString)]
#[strum(serialize_all = "snake_case")]
enum Engine {
    Kvs,
    Sled,
}

#[derive(FromArgs)]
/// Kvs server
struct Args {
    /// print version information
    #[argh(switch, short = 'V')]
    version: bool,

    /// IP:port, used to connect server
    #[argh(option)]
    addr: Option<String>,

    /// specify an engine [possible values: kvs, sled]
    #[argh(option)]
    engine: Option<Engine>,
}


fn main() {
    let args: Args = argh::from_env();

    if args.version {
        println!("kvs-server {}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }

    let addr = args.addr.unwrap_or("127.0.0.1:4000".to_string());
    let socket_addr: SocketAddr = match addr.parse() {
        Ok(val) => val,
        Err(_e) => {
            println!("The address {} is invalid", &addr);
            exit(-1);
        }
    };

    let engine = match args.engine {
        Some(engine_arg) => {
            match engine_arg {
                Engine::Kvs => Engine::Kvs.to_string(),
                Engine::Sled => Engine::Sled.to_string()
            }
        }
        None => "kvs".to_string()
    };

    let plain = slog_term::PlainSyncDecorator::new(std::io::stderr());
    let logger = slog::Logger::root(
        slog_term::FullFormat::new(plain).build().fuse(),
        o!("src" => PushFnValue(|r: &Record, ser: PushFnValueSerializer| {
            ser.emit(format_args!("{}:{}", r.file(), r.line()))
        })),
    );
    let _guard = slog_scope::set_global_logger(logger);

    info!("Server version: {}", env!("CARGO_PKG_VERSION"));
    info!("Run with {} engine", engine);
    info!("Listening on {}", addr);

    let mut server = KvsServer::new(socket_addr, engine);
    server.handle_connection();
}
