// Lightweight Gorilla-style floating point encoder/decoder.
// This is a simplified implementation (no reuse of previous block header),
// but compatible between `encode` and `decode` here.

struct BitWriter {
    buf: Vec<u8>,
    cur: u8,
    bits_filled: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self { buf: Vec::new(), cur: 0, bits_filled: 0 }
    }

    fn push_bit(&mut self, bit: u8) {
        if bit != 0 {
            self.cur |= 1 << (7 - self.bits_filled);
        }
        self.bits_filled += 1;
        if self.bits_filled == 8 {
            self.buf.push(self.cur);
            self.cur = 0;
            self.bits_filled = 0;
        }
    }

    fn write_bits(&mut self, mut value: u64, bits: usize) {
        for i in (0..bits).rev() {
            let b = ((value >> i) & 1) as u8;
            self.push_bit(b);
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.bits_filled > 0 {
            self.buf.push(self.cur);
        }
        self.buf
    }
}

struct BitReader<'a> {
    buf: &'a [u8],
    bit_pos: usize, // 0..(len*8)
}

impl<'a> BitReader<'a> {
    fn new(buf: &'a [u8]) -> Self { Self { buf, bit_pos: 0 } }

    fn remaining_bits(&self) -> usize { self.buf.len() * 8 - self.bit_pos }

    fn read_bit(&mut self) -> Option<u8> {
        if self.bit_pos >= self.buf.len() * 8 { return None; }
        let byte_idx = self.bit_pos / 8;
        let bit_idx = 7 - (self.bit_pos % 8);
        let b = (self.buf[byte_idx] >> bit_idx) & 1;
        self.bit_pos += 1;
        Some(b)
    }

    fn read_bits(&mut self, bits: usize) -> Option<u64> {
        if self.remaining_bits() < bits { return None; }
        let mut v = 0u64;
        for _ in 0..bits {
            v = (v << 1) | (self.read_bit()? as u64);
        }
        Some(v)
    }
}

pub fn encode(values: &[f64]) -> Vec<u8> {
    if values.is_empty() {
        return Vec::new();
    }

    let mut w = BitWriter::new();

    // write first value verbatim (64 bits)
    let first_bits = values[0].to_bits();
    w.write_bits(first_bits, 64);
    let mut prev = first_bits;

    for &v in &values[1..] {
        let cur = v.to_bits();
        let x = prev ^ cur;
        if x == 0 {
            // flag 0 -> same value
            w.push_bit(0);
        } else {
            w.push_bit(1); // non-zero xor
            let lz = x.leading_zeros() as usize; // 0..64
            let tz = x.trailing_zeros() as usize;
            let siglen = 64 - lz - tz; // >0
            // encode lz in 6 bits (0..63), siglen-1 in 6 bits (0..63)
            let lz_enc = (lz as u64) & 0x3f;
            let sl_enc = ((siglen - 1) as u64) & 0x3f;
            w.write_bits(lz_enc, 6);
            w.write_bits(sl_enc, 6);
            let sigbits = (x >> tz) & ((1u128 << siglen) - 1) as u64;
            w.write_bits(sigbits, siglen);
        }
        prev = cur;
    }

    w.finish()
}

pub fn decode(data: &[u8]) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut r = BitReader::new(data);
    // read first 64 bits
    let first = match r.read_bits(64) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    out.push(f64::from_bits(first));
    let mut prev = first;

    while r.remaining_bits() > 0 {
        // need at least 1 bit
        let flag = match r.read_bit() {
            Some(b) => b,
            None => break,
        };
        if flag == 0 {
            out.push(f64::from_bits(prev));
        } else {
            // need 12 bits for lz and siglen
            let lz = match r.read_bits(6) {
                Some(v) => v as usize,
                None => break,
            };
            let slm1 = match r.read_bits(6) {
                Some(v) => v as usize,
                None => break,
            };
            let siglen = slm1 + 1;
            if r.remaining_bits() < siglen { break; }
            let sig = r.read_bits(siglen).unwrap();
            let tz = 64 - lz - siglen;
            let xor = sig << tz;
            let cur = prev ^ xor;
            out.push(f64::from_bits(cur));
            prev = cur;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simple() {
        let vals = vec![0.0f64, 0.0, 1.0, 1.0000001, -5.5, -5.5, 12345.6789];
        let enc = encode(&vals);
        let dec = decode(&enc);
        assert_eq!(vals.len(), dec.len());
        for (a, b) in vals.iter().zip(dec.iter()) {
            if a.is_nan() {
                assert!(b.is_nan());
            } else {
                assert_eq!(a.to_bits(), b.to_bits());
            }
        }
    }

    #[test]
    fn roundtrip_empty() {
        let v: Vec<f64> = vec![];
        let enc = encode(&v);
        let dec = decode(&enc);
        assert!(dec.is_empty());
    }
}
