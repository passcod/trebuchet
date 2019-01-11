use byteorder::{ByteOrder, LittleEndian};

pub fn parse(raw: &[u8]) -> Option<(&[u8], &[u8])> {
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

    let header = &raw[5..(5 + size)];
    let body = &raw[(6 + size)..];

    Some((header, body))
}
