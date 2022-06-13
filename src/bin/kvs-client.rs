extern crate anyhow;

use std::net::SocketAddr;
use std::process::exit;

use anyhow::Result;
use argh::FromArgs;

use kvs::KvsClient;

#[derive(FromArgs, PartialEq, Debug)]
/// Kvs client
struct Args {
    #[argh(subcommand)]
    subcommand: Option<SubCommandEnum>,

    #[argh(option)]
    /// IP:port, used to connect server
    addr: Option<String>,
    #[argh(switch, short = 'V')]
    /// print version information
    version: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommandEnum {
    Get(GetSubCommand),
    Set(SetSubCommand),
    Rm(RmSubCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Get the string value of a given string key
#[argh(subcommand, name = "get")]
struct GetSubCommand {
    #[argh(positional)]
    /// key
    key: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Set the value of a string key to a string
#[argh(subcommand, name = "set")]
struct SetSubCommand {
    #[argh(positional)]
    /// key
    key: String,

    #[argh(positional)]
    /// value
    value: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Remove a given key
#[argh(subcommand, name = "rm")]
struct RmSubCommand {
    #[argh(positional)]
    /// key
    key: String,
}


fn main() -> Result<()> {
    let args: Args = argh::from_env();

    if args.version {
        println!("kvs-client {}", env!("CARGO_PKG_VERSION"));
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
    let client = KvsClient::new(socket_addr);

    let subcommand = match args.subcommand {
        Some(command) => command,
        None => {
            exit(-1);
        }
    };
    match subcommand {
        SubCommandEnum::Get(command_arg) => {
            let key = command_arg.key;
            let val = match client.get(&key)? {
                Some(val) => val,
                None => {
                    println!("Key not found");
                    exit(0);
                }
            };
            println!("{}", val);
        }
        SubCommandEnum::Rm(command_arg) => {
            let key = command_arg.key;
            if !client.is_key_exist(&key)? {
                eprintln!("Key not found");
                exit(-1);
            }
            client.remove(&key)?;
        }
        SubCommandEnum::Set(command_arg) => {
            let key = command_arg.key;
            let value = command_arg.value;
            client.set(&key, &value)?;
        }
    };

    Ok(())
}
