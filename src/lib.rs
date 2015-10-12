#[macro_use]
extern crate log;
extern crate hyper;
extern crate rustc_serialize;

use std::io::Read;
use std::sync::mpsc::Sender;

use hyper::Client;
use hyper::error::Error;

use rustc_serialize::json;
use rustc_serialize::Decodable;

#[derive(Debug)]
pub enum SSEError {
    ClientError(hyper::error::Error),
    ReadBody,
    StringConv,
    JsonDecode,
    TxFailed,
}


pub struct SSE {
    return_on_err: bool,
    target:       String,
}

impl SSE {
    pub fn new(target: String, return_on_err: bool) -> SSE {
        SSE {
            return_on_err: return_on_err,
            target:        target,
        }
    }

    pub fn listen<T: Decodable>(&self, tx: Sender<T>) -> Result<(), SSEError> {
        let client = Client::new();
        let mut res = match client.get(&self.target).send() {
            Ok(r)  => r,
            Err(e) => return Err(SSEError::ClientError(e)),
        };

        loop {

            let mut out = String::new();

            loop {
                let mut buf = [0; 512];
                match res.read(&mut buf) {
                    Ok(c)  => c,
                    Err(e) => {
                        debug!("{:?}", e);
                        if self.return_on_err {
                            return Err(SSEError::ReadBody)
                        }
                        break
                    },
                };
                let part = match String::from_utf8(buf.to_vec()) {
                    Ok(p)  => p,
                    Err(e) => {
                        debug!("{:?}", e);
                        if self.return_on_err {
                            return Err(SSEError::StringConv)
                        }
                        break
                    },
                };
                out = out + part.as_ref();
                if out.contains("\n") {
                    break
                }
            }

            let i = out.find("\n").unwrap();
            let e: T = match json::decode(&out[..i]) {
                Ok(ev) => ev,
                Err(e) => {
                    debug!("{:?}", e);
                    if self.return_on_err {
                        return Err(SSEError::JsonDecode)
                    }
                    continue
                },
            };

            match tx.send(e) {
                Ok(_)  => (),
                Err(e) => {
                    debug!("{:?}", e);
                    if self.return_on_err {
                        return Err(SSEError::TxFailed)
                    }
                    continue
                },
            };
        }
    }

}
