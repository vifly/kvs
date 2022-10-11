use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::exit;

use serde_json::{Deserializer, to_writer};
use slog_scope::{debug, error};

use crate::{KvsEngine, Request, Response, Result};

pub struct KvsServer<E: KvsEngine> {
    addr: SocketAddr,
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    pub fn new(addr: SocketAddr, engine: E) -> Self {
        KvsServer { addr, engine }
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
