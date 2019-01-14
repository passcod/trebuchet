use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use jsonrpc_core::{
    Call, Id, MethodCall, Notification, Output, Params, Request, Response, Value, Version,
};
use std::io::Write;

/// Either of a JSON-RPC Request or Response.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Rpc {
    Request(Request),
    Response(Response),
}

/// Parses a regular string message as an RPC Message.
pub fn parse_plain(string: &str) -> Option<Rpc> {
    let len = string.len();
    if len < 30 {
        // 30 is the smallest possible JSON-RPC message (notification, single-letter method)
        warn!("invalid plain message length ({}) received, ignoring", len);
        return None;
    }

    match serde_json::from_str::<Rpc>(string) {
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
///
/// The binary data is expected to contain:
///  - **One byte** as version (expected to be `1`),
///  - **Four bytes** as header length, little-endian u32,
///  - **Header length bytes** as header, UTF-8 string parsed as JSON-RPC,
///  - **Remainder** as body, raw binary data.
///
/// for a minimum length of 35 bytes.
///
/// The handling of the binary data depends on the type of the JSON-RPC message:
///  - If a **Notification**, treat the same as a Request.
///  - If a **Request**, add as parameter:
///     + if the parameter structure is an **Array**, simply append the data as a last item,
///     + if the parameter structure is an **Object**, insert the data under the `.raw` key.
///  - If a **Response**, vary based on `result` or `error` type:
///     + if a **Structured**, proceed as for Request,
///     + if a **Primitive**, replace with an Array containing `[original, raw]`.
///
/// Only single JSON-RPC calls and responses are supported, not batches.
pub fn parse_binary(raw: &[u8]) -> Option<Rpc> {
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

    let body: Vec<u8> = raw[(6 + size)..].into();
    let body: Value = body.into();

    let header = match std::str::from_utf8(&raw[5..(5 + size)]) {
        Err(err) => {
            warn!("invalid raw message header (utf-8 parsing error): {}", err);
            return None;
        }
        Ok(s) => s,
    };

    match serde_json::from_str::<Rpc>(header) {
        Err(err) => {
            warn!("invalid raw message header: {}", err);
            None
        }
        Ok(Rpc::Request(Request::Single(mut req))) => {
            match req {
                Call::Invalid { .. } => {
                    warn!("invalid raw message header: invalid call");
                    return None;
                }
                Call::MethodCall(ref mut meth) => {
                    meth.params = append_params(&meth.params, body);
                }
                Call::Notification(ref mut note) => {
                    note.params = append_params(&note.params, body);
                }
            };

            trace!("valid binary request parsed: {:?}", req);
            Some(Rpc::Request(Request::Single(req)))
        }
        Ok(Rpc::Response(Response::Single(mut res))) => {
            match res {
                Output::Success(ref mut succ) => {
                    succ.result = append_value(&succ.result, body);
                }
                Output::Failure(ref mut fail) => {
                    fail.error.data = Some(match fail.error.data {
                        None => body,
                        Some(ref val) => append_value(&val, body),
                    });
                }
            };

            trace!("valid binary response parsed: {:?}", res);
            Some(Rpc::Response(Response::Single(res)))
        }
        _ => {
            warn!("invalid raw message header: batch");
            None
        }
    }
}

fn append_params(params: &Params, body: Value) -> Params {
    match params {
        Params::None => Params::None,
        Params::Array(arr) => {
            let mut arr = arr.clone();
            arr.push(body);
            Params::Array(arr)
        }
        Params::Map(map) => {
            let mut map = map.clone();
            map.insert(".raw".into(), body);
            Params::Map(map)
        }
    }
}

fn append_value(value: &Value, body: Value) -> Value {
    match value {
        Value::Array(arr) => {
            let mut arr = arr.clone();
            arr.push(body);
            Value::Array(arr)
        }
        Value::Object(map) => {
            let mut map = map.clone();
            map.insert(".raw".into(), body);
            Value::Object(map)
        }
        val @ _ => {
            let val = val.clone();
            Value::Array(vec![val, body])
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
pub fn add_binary(header: String, binary: &[u8]) -> Vec<u8> {
    let headlen = header.len();
    let mut buf = Vec::with_capacity(5 + headlen + binary.len());
    buf.write_all(&[1]).unwrap(); // version
    buf.write_u32::<LittleEndian>(headlen as u32).unwrap();
    buf.write_all(header.as_bytes()).unwrap();
    buf.write_all(binary).unwrap();
    buf
}
