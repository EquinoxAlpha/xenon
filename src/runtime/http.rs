use std::{collections::HashMap, io::Read};

use log::{error, info};
use rhai::{Dynamic, Engine};

pub fn register_functions(engine: &mut Engine) {
    engine.register_fn("parse_http_request", |request: Vec<u8>| -> Dynamic {
        let mut bytes = request.iter().peekable();

        let method = bytes.by_ref().take_while(|b| **b != b' ').map(|b| *b as char).collect::<String>();
        let path = bytes.by_ref().take_while(|b| **b != b' ').map(|b| *b as char).collect::<String>();
        let version = bytes.by_ref().take_while(|b| **b != b'\r').map(|b| *b as char).collect::<String>();
        bytes.next(); // skip '\n'

        let mut headers = HashMap::new();
        let mut header_buffer = Vec::new();
        loop {
            let Some(byte) = bytes.next() else {
                return Dynamic::UNIT;
            };

            if *byte == b'\r' && bytes.peek() == Some(&&b'\n') {
                let header = String::from_utf8(header_buffer.clone()).unwrap();
                let split = header.splitn(2, |c: char| c == ':').collect::<Vec<_>>();
                headers.insert(split[0].to_string().to_lowercase(), split[1][1..].to_string());
                header_buffer.clear();
                bytes.next(); // skip '\n'

                if bytes.peek() == Some(&&b'\r') && bytes.nth(1) == Some(&b'\n') {
                    // end of headers
                    break;
                }
            } else {
                header_buffer.push(*byte);
            }
        }

        let mut body = bytes.map(|b| *b).collect::<Vec<_>>();

        if let Some(content_length) = headers.get("content-length") {
            if let Ok(length) = content_length.parse::<usize>() {
                body.truncate(length);
            }
        }

        if body.len() > 10 {
            info!("Body: {:02x?}", &body[..10]);
        }

        if let Some(content_encoding) = headers.get("content-encoding") {
            if content_encoding == "gzip" {
                info!("Decoding gzip");
                let cl = body.clone();
                let mut decoder = flate2::read::GzDecoder::new(cl.as_slice());
                if decoder.read_to_end(&mut body).is_err() {
                    error!("Failed to decode content");
                    return Dynamic::UNIT;
                }
            }
        }

        let mut map = rhai::Map::new();
        map.insert("method".into(), method.into());
        map.insert("path".into(), path.into());
        map.insert("version".into(), version.into());
        map.insert("headers".into(), headers.into());
        map.insert("body".into(), rhai::Blob::from(body).into());

        map.into()
    });
}
