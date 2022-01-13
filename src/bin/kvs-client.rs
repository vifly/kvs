extern crate anyhow;
extern crate clap;

use std::net::SocketAddr;
use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, load_yaml, App};
use kvs::KvStore;
use std::process::exit;

fn main() -> Result<()> {
    let yaml = load_yaml!("client-cli.yml");
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
        },
    };

    if let Some(ref matches) = m.subcommand_matches("set") {
        if matches.is_present("key") && matches.is_present("value") {
            let mut kvs = KvStore::open("./")?;
            let key = matches.value_of("key").unwrap().to_string();
            let value = matches.value_of("value").unwrap().to_string();
            kvs.set(key, value)?;
        } else {
            exit(-1);
        }
    } else if let Some(ref matches) = m.subcommand_matches("get") {
        if matches.is_present("key") {
            let key = matches.value_of("key").unwrap().to_string();
            let kvs = KvStore::open("./")?;
            let val = match kvs.get(key)? {
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
            let mut kvs = KvStore::open("./")?;
            if !kvs.is_key_exist(&key) {
                println!("Key not found");
                exit(-1);
            }
            kvs.remove(key)?;
        } else {
            exit(-1);
        }
    } else {
        exit(-1);
    }

    Ok(())
}
