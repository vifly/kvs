use serde_json::{Deserializer, to_writer};
use slog_scope::{debug, error};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::exit;

use crate::{KvsEngine, KvStore, Request, Response, Result, SledKvsEngine};

pub struct KvsServer {
    addr: SocketAddr,
    engine: Box<dyn KvsEngine>,
}

fn get_engine_name(path: impl Into<PathBuf>) -> Result<Option<String>> {
    let path = path.into().join("engine");
    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        return Ok(Some(contents));
    }
    Ok(None)
}

fn write_engine(engine: &str, path: impl Into<PathBuf>) -> Result<()> {
    let path = path.into().join("engine");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(path)?;
    file.write_all(engine.as_bytes())?;

    Ok(())
}

impl KvsServer {
    pub fn new(addr: SocketAddr, engine: String) -> KvsServer {
        match get_engine_name("./") {
            Ok(res) => {
                if let Some(val) = res {
                    if val.ne(&engine) {
                        error!("Wrong engine, before: {}, now: {}", val, engine);
                        exit(-1);
                    }
                }
            },
            Err(e) => {
                error!("Can't get engine record: {}", e);
                exit(-1);
            }
        };
        if engine.eq("kvs") {
            let kvs = KvStore::open("./").unwrap_or_else(|e| {
                error!("Can't open KvStore: {}", e);
                exit(-1);
            });
            write_engine(&engine, "./").unwrap_or_else(|e| {
                error!("Can't write engine record: {}", e);
                exit(-1);
            });
            KvsServer { addr, engine: Box::new(kvs) }
        } else {
            let sled = SledKvsEngine::open("./").unwrap_or_else(|e| {
                error!("Can't open Sled: {}", e);
                exit(-1);
            });
            write_engine(&engine, "./").unwrap_or_else(|e| {
                error!("Can't write engine record: {}", e);
                exit(-1);
            });
            KvsServer { addr, engine: Box::new(sled) }
        }
    }

    pub fn handle_connection(&mut self) {
        let listener = match TcpListener::bind(self.addr) {
            Ok(listener) => listener,
            Err(e) => {
                error!("Can't bind address. addr: {}, error: {}", self.addr, e);
                exit(-1);
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    debug!("Receive connection.");
                    self.handle_stream(&stream);
                }
                Err(e) => error!("Connection error: {}", e),
            }
        }

        // close the socket server
        drop(listener);
    }

    fn handle_stream(&mut self, stream: &TcpStream) {
        let reader = BufReader::new(stream);
        let request_reader = Deserializer::from_reader(reader).into_iter::<Request>();
        for command in request_reader {
            match command {
                Ok(command) => {
                    match self.send_resp(stream, &command) {
                        Ok(_) => debug!("Send response."),
                        Err(e) => error!("Failed to send response: {}", e)
                    };
                }
                Err(e) => {
                    error!("Can't parse request: {}", e);
                }
            };
        }
    }

    fn send_resp(&mut self, stream: &TcpStream, request: &Request) -> Result<()> {
        let mut writer = BufWriter::new(stream);


        match request {
            Request::Set { key, value } => {
                match self.engine.set(key.to_string(), value.to_string()) {
                    Ok(_) => {
                        to_writer(&mut writer, &Response::new(true, "".to_string()))?;
                    }
                    Err(e) => {
                        to_writer(&mut writer, &Response::new(false, e.to_string()))?;
                    }
                };
            }
            Request::Get { key } => {
                match self.engine.get(key.to_string()) {
                    Ok(val) => {
                        let value = val.unwrap_or("".to_string());
                        to_writer(&mut writer, &Response::new(true, value))?;
                    }
                    Err(e) => {
                        to_writer(&mut writer, &Response::new(false, e.to_string()))?;
                    }
                };
            }
            Request::Rm { key } => {
                match self.engine.remove(key.to_string()) {
                    Ok(_) => {
                        to_writer(&mut writer, &Response::new(true, "".to_string()))?;
                    }
                    Err(e) => {
                        to_writer(&mut writer, &Response::new(false, e.to_string()))?;
                    }
                };
            }
        }
        writer.flush()?;

        Ok(())
    }
}
