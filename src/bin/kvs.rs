extern crate anyhow;
extern crate clap;

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, load_yaml, App};
use kvs::KvStore;
use std::process::exit;

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .get_matches();

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
                    exit(0);
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
