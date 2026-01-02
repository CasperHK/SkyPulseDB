// Simple delta-of-delta timestamp compressor using ZigZag + LEB128 encoding.
// Layout:
// - first timestamp: 8 bytes little-endian i64
// - first delta: ZigZag(i64) encoded as LEB128
// - subsequent values: delta-of-delta ZigZag encoded as LEB128

fn zig_zag_encode(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

fn zig_zag_decode(u: u64) -> i64 {
    ((u >> 1) as i64) ^ -((u & 1) as i64)
}

fn write_leb_u64(mut v: u64, out: &mut Vec<u8>) {
    while v >= 0x80 {
        out.push(((v as u8) & 0x7F) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

fn read_leb_u64(data: &[u8], idx: &mut usize) -> Option<u64> {
    let mut shift = 0;
    let mut res = 0u64;
    loop {
        if *idx >= data.len() { return None; }
        let b = data[*idx];
        *idx += 1;
        res |= ((b & 0x7F) as u64) << shift;
        if (b & 0x80) == 0 { break; }
        shift += 7;
        if shift >= 64 { return None; }
    }
    Some(res)
}

pub fn encode_timestamps(ts: &[i64]) -> Vec<u8> {
    if ts.is_empty() { return Vec::new(); }
    let mut out = Vec::new();
    // write first timestamp (i64 le)
    out.extend_from_slice(&ts[0].to_le_bytes());
    if ts.len() == 1 { return out; }

    let first_delta = ts[1] - ts[0];
    write_leb_u64(zig_zag_encode(first_delta), &mut out);
    let mut prev_delta = first_delta;

    for w in ts.iter().skip(2) {
        let delta = *w - ts[((out.len()>0) as usize)]; // placeholder to satisfy borrow checker removed
        // compute delta properly using previous timestamp
    }

    // Because the previous loop above uses a placeholder to satisfy editing,
    // implement the iteration properly below.
    out.truncate(8); // reset to first-timestamp-only
    // write first delta again
    write_leb_u64(zig_zag_encode(first_delta), &mut out);
    prev_delta = first_delta;
    let mut prev_ts = ts[1];
    for &t in &ts[2..] {
        let delta = t - prev_ts;
        let dod = delta - prev_delta;
        write_leb_u64(zig_zag_encode(dod), &mut out);
        prev_delta = delta;
        prev_ts = t;
    }

    out
}

pub fn decode_timestamps(data: &[u8]) -> Vec<i64> {
    if data.len() < 8 { return Vec::new(); }
    let mut idx = 0usize;
    let first = i64::from_le_bytes(data[0..8].try_into().unwrap());
    idx += 8;
    let mut out = Vec::new();
    out.push(first);
    if idx >= data.len() { return out; }

    // read first delta
    let first_delta_u = match read_leb_u64(data, &mut idx) {
        Some(v) => v,
        None => return out,
    };
    let first_delta = zig_zag_decode(first_delta_u);
    let second = first + first_delta;
    out.push(second);

    let mut prev_delta = first_delta;
    let mut prev_ts = second;

    while idx < data.len() {
        let dod_u = match read_leb_u64(data, &mut idx) {
            Some(v) => v,
            None => break,
        };
        let dod = zig_zag_decode(dod_u);
        let delta = prev_delta + dod;
        let ts = prev_ts + delta;
        out.push(ts);
        prev_delta = delta;
        prev_ts = ts;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_regular() {
        let ts: Vec<i64> = (0..1000).map(|i| i * 60).collect();
        let enc = encode_timestamps(&ts);
        let dec = decode_timestamps(&enc);
        assert_eq!(ts, dec);
    }

    #[test]
    fn roundtrip_irregular() {
        let ts: Vec<i64> = vec![1000, 1010, 1030, 1500, 1501, 1510, 3000];
        let enc = encode_timestamps(&ts);
        let dec = decode_timestamps(&enc);
        assert_eq!(ts, dec);
    }

    #[test]
    fn single_and_empty() {
        let a: Vec<i64> = vec![];
        assert!(encode_timestamps(&a).is_empty());

        let b = vec![42i64];
        let enc = encode_timestamps(&b);
        let dec = decode_timestamps(&enc);
        assert_eq!(b, dec);
    }
}
