use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use jsonrpc_core::{
    Call, Id, MethodCall, Notification, Output, Params, Request, Response, Value, Version,
};
use log::{trace, warn};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;

/// Either of a JSON-RPC Request or Response.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RpcMessage {
    Response(Response), // Order is important for deserialisation!
    Request(Request),   // Requests can leave off stuff, so try response first
}

/// Parses a `ws::Message` as an RPC Message.
pub fn parse_ws(msg: ws::Message) -> Option<RpcMessage> {
    match msg {
        ws::Message::Text(string) => {
            trace!("string message received: {:?}", string);
            parse_plain(&string)
        }
        ws::Message::Binary(raw) => {
            trace!("raw message received: {:?}", raw);
            parse_binary(&raw)
        }
    }
}

/// Parses a regular string message as an RPC Message.
pub fn parse_plain(string: &str) -> Option<RpcMessage> {
    let len = string.len();
    if len < 30 {
        // 30 is the smallest possible JSON-RPC message (notification, single-letter method)
        warn!("invalid plain message length ({}) received, ignoring", len);
        return None;
    }

    match serde_json::from_str::<RpcMessage>(string) {
        Err(err) => {
            warn!("invalid plain message: {}", err);
            None
        }
        Ok(rpc) => {
            trace!("valid plain message parsed: {:?}", rpc);
            Some(rpc)
        }
    }
}

/// Parses an extended binary message as an RPC Message.
///
/// This is an Armstrong extension to JSON-RPC to facilitate passing binary data along RPC messages.
/// The format is loosely inspired by Khronos' binary GLTF.
///
/// The binary data is expected to contain:
///  - **One byte** as version (expected to be `1`),
///  - **Four bytes** as header length, little-endian u32,
///  - **Header length bytes** as header, UTF-8 string parsed as JSON-RPC,
///  - **One byte** as the number of chunks, omitted if none,
///  - **Remainder** as raw binary chunks, each organised such:
///    - **Four bytes** as chunk length, little-endian u32,
///    - **Chunk length bytes** as a chunk, raw binary data,
///
/// for a minimum length of 35 bytes.
///
/// The handling of the binary data depends on the type of the JSON-RPC message:
///  - If a **Notification**, treat the same as a Request.
///  - If a **Request**, add as parameter:
///     + if the parameter structure is an **Array**, simply append each chunk,
///     + if the parameter structure is an **Object**, insert the data as an array under the `.raw` key.
///  - If a **Response**, vary based on `result` or `error` type:
///     + if a **Structured**, proceed as for Request,
///     + if a **Primitive**, replace with an Array containing `[original, ...chunks]`.
///
/// Only single JSON-RPC calls and responses are supported, not batches.
pub fn parse_binary(raw: &[u8]) -> Option<RpcMessage> {
    let len = raw.len();
    if len < 35 {
        // 30 is the smallest possible JSON-RPC message (notification, single-letter method)
        // + 4 bytes for the header length prefix
        // + 1 byte for the version prefix
        warn!("invalid raw message length ({}) received, ignoring", len);
        return None;
    }

    let version = &raw[0];
    if version != &1 {
        warn!(
            "invalid raw message version ({}) received, ignoring",
            version
        );
        return None;
    }

    // parse as header size + header + body
    let size = LittleEndian::read_u32(&raw[1..4]) as usize;
    if size + 5 > raw.len() {
        warn!(
            "invalid raw message header size ({}) received, ignoring",
            size
        );
        return None;
    }

    let chunks = parse_chunks(&raw[(6 + size)..]);

    let header = match std::str::from_utf8(&raw[5..(5 + size)]) {
        Err(err) => {
            warn!("invalid raw message header (utf-8 parsing error): {}", err);
            return None;
        }
        Ok(s) => s,
    };

    match serde_json::from_str::<RpcMessage>(header) {
        Err(err) => {
            warn!("invalid raw message header: {}", err);
            None
        }
        Ok(RpcMessage::Request(Request::Single(mut req))) => {
            match req {
                Call::Invalid { .. } => {
                    warn!("invalid raw message header: invalid call");
                    return None;
                }
                Call::MethodCall(ref mut meth) => {
                    meth.params = append_params(&meth.params, chunks);
                }
                Call::Notification(ref mut note) => {
                    note.params = append_params(&note.params, chunks);
                }
            };

            trace!("valid binary request parsed: {:?}", req);
            Some(RpcMessage::Request(Request::Single(req)))
        }
        Ok(RpcMessage::Response(Response::Single(mut res))) => {
            match res {
                Output::Success(ref mut succ) => {
                    succ.result = append_values(&succ.result, chunks);
                }
                Output::Failure(ref mut fail) => {
                    fail.error.data = Some(match fail.error.data {
                        None => Value::Array(chunks),
                        Some(ref val) => append_values(&val, chunks),
                    });
                }
            };

            trace!("valid binary response parsed: {:?}", res);
            Some(RpcMessage::Response(Response::Single(res)))
        }
        _ => {
            warn!("invalid raw message header: batch");
            None
        }
    }
}

fn parse_chunks(data: &[u8]) -> Vec<Value> {
    if data.is_empty() {
        return Vec::new();
    }

    let overall = data.len();

    let mut n = data[0] as usize;
    let mut chunks: Vec<&[u8]> = Vec::with_capacity(n);
    let mut cursor = 1;

    while n > 0 {
        n -= 1;

        if cursor + 5 > overall {
            warn!("invalid raw message: chunks are too short");
            return Vec::new();
        }

        let len = LittleEndian::read_u32(&data[cursor..(cursor + 3)]) as usize;
        cursor += 4;

        chunks.push(&data[cursor..(cursor + len - 1)]);
        cursor += len;
    }

    chunks.into_iter().map(|c| {
        let v: Vec<u8> = c.into();
        v.into()
    }).collect()
}

fn append_params(params: &Params, chunks: Vec<Value>) -> Params {
    match params {
        Params::None => if chunks.is_empty() {
            Params::None
        } else {
            Params::Array(chunks)
        },
        Params::Array(arr) => {
            let mut arr = arr.clone();
            arr.extend(chunks);
            Params::Array(arr)
        }
        Params::Map(map) => {
            let mut map = map.clone();
            map.insert(".raw".into(), Value::Array(chunks));
            Params::Map(map)
        }
    }
}

fn append_values(value: &Value, mut chunks: Vec<Value>) -> Value {
    match value {
        Value::Array(arr) => {
            let mut arr = arr.clone();
            arr.extend(chunks);
            Value::Array(arr)
        }
        Value::Object(map) => {
            let mut map = map.clone();
            map.insert(".raw".into(), Value::Array(chunks));
            Value::Object(map)
        }
        val => {
            chunks.insert(0, val.clone());
            Value::Array(chunks)
        }
    }
}

pub fn notification(method: String, params: Params) -> String {
    json!(Request::Single(
        Notification {
            jsonrpc: Some(Version::V2),
            method,
            params,
        }
        .into(),
    ))
    .to_string()
}

pub fn methodcall(method: String, params: Params, id: Id) -> String {
    json!(Request::Single(
        MethodCall {
            jsonrpc: Some(Version::V2),
            method,
            params,
            id,
        }
        .into(),
    ))
    .to_string()
}

#[allow(clippy::cast_possible_truncation)]
pub fn add_chunks(header: String, chunks: &[&[u8]]) -> Vec<u8> {
    let headlen = header.len();
    let rawlen = chunks.iter().fold(1, |sum, c| sum + c.len());
    let mut buf = Vec::with_capacity(5 + headlen + rawlen);
    buf.write_all(&[1]).unwrap(); // version
    buf.write_u32::<LittleEndian>(headlen as u32).unwrap();
    buf.write_all(header.as_bytes()).unwrap();
    buf.write_u8(chunks.len() as u8).unwrap();
    for chunk in chunks {
        buf.write_u32::<LittleEndian>(chunk.len() as u32).unwrap();
        buf.write_all(chunk).unwrap();
    }
    buf
}
