use std::net::{SocketAddr, TcpStream};
use std::io::{BufReader, BufWriter, Write};
use serde::Deserialize;
use serde_json::{Deserializer, to_writer};
use crate::{KvsError, Request, Response, Result};

pub struct KvsClient {
    addr: SocketAddr,
}

impl KvsClient {
    pub fn new(addr: SocketAddr) -> KvsClient {
        KvsClient { addr }
    }

    fn send_command(&self, request: Request) -> Result<Response> {
        let stream = TcpStream::connect(self.addr)?;
        let cloned_stream = stream.try_clone()?;
        let mut reader = Deserializer::from_reader(BufReader::new(stream));
        let mut writer = BufWriter::new(cloned_stream);
        to_writer(&mut writer, &request)?;
        writer.flush()?;

        let resp = Response::deserialize(&mut reader)?;
        Ok(resp)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let request = Request::Set { key: key.to_string(), value: value.to_string() };
        let resp = self.send_command(request)?;
        if resp.is_ok {
            Ok(())
        } else {
            Err(KvsError::ServerRespError(resp.data))
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let request = Request::Get { key: key.to_string() };
        let resp = self.send_command(request)?;
        if resp.is_ok {
            if resp.data.is_empty() {
                Ok(None)
            } else {
                Ok(Some(resp.data))
            }
        } else {
            Err(KvsError::ServerRespError(resp.data))
        }
    }

    pub fn remove(&self, key: &str) -> Result<()> {
        let request = Request::Rm { key: key.to_string() };
        let resp = self.send_command(request)?;
        if resp.is_ok {
            Ok(())
        } else {
            Err(KvsError::ServerRespError(resp.data))
        }
    }

    pub fn is_key_exist(&self, key: &str) -> Result<bool> {
        let result = self.get(key)?.is_some();
        Ok(result)
    }
}
