extern crate anyhow;
extern crate clap;

use std::net::SocketAddr;
use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, load_yaml, App};
use kvs::KvsClient;
use std::process::exit;

fn main() -> Result<()> {
    let yaml = load_yaml!("client-cli.yml");
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
        },
    };
    let client = KvsClient::new(socket_addr);

    if let Some(ref matches) = m.subcommand_matches("set") {
        if matches.is_present("key") && matches.is_present("value") {
            let key = matches.value_of("key").unwrap().to_string();
            let value = matches.value_of("value").unwrap().to_string();
            client.set(&key, &value)?;
        } else {
            exit(-1);
        }
    } else if let Some(ref matches) = m.subcommand_matches("get") {
        if matches.is_present("key") {
            let key = matches.value_of("key").unwrap().to_string();
            let val = match client.get(&key)? {
                Some(val) => val,
                None => {
                    println!("Key not found");
                    exit(-1);
                }
            };
            println!("{}", val);
        } else {
            exit(-1);
        }
    } else if let Some(ref matches) = m.subcommand_matches("rm") {
        if matches.is_present("key") {
            let key = matches.value_of("key").unwrap().to_string();
            if !client.is_key_exist(&key)? {
                println!("Key not found");
                exit(-1);
            }
            client.remove(&key)?;
        } else {
            exit(-1);
        }
    } else {
        exit(-1);
    }

    Ok(())
}
