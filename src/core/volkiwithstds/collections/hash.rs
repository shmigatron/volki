//! SipHash-1-3 hasher — implements core::hash::Hasher.

use core::hash::Hasher;

/// SipHash-1-3 state.
pub struct SipHasher {
    v0: u64,
    v1: u64,
    v2: u64,
    v3: u64,
    buf: [u8; 8],
    buf_len: usize,
    length: usize,
}

impl SipHasher {
    /// Create a new SipHasher with default keys.
    pub fn new() -> Self {
        Self::with_keys(0x0706050403020100, 0x0f0e0d0c0b0a0908)
    }

    /// Create a SipHasher with specific keys.
    pub fn with_keys(key0: u64, key1: u64) -> Self {
        Self {
            v0: key0 ^ 0x736f6d6570736575,
            v1: key1 ^ 0x646f72616e646f6d,
            v2: key0 ^ 0x6c7967656e657261,
            v3: key1 ^ 0x7465646279746573,
            buf: [0; 8],
            buf_len: 0,
            length: 0,
        }
    }

    #[inline]
    fn sip_round(&mut self) {
        self.v0 = self.v0.wrapping_add(self.v1);
        self.v1 = self.v1.rotate_left(13);
        self.v1 ^= self.v0;
        self.v0 = self.v0.rotate_left(32);
        self.v2 = self.v2.wrapping_add(self.v3);
        self.v3 = self.v3.rotate_left(16);
        self.v3 ^= self.v2;
        self.v0 = self.v0.wrapping_add(self.v3);
        self.v3 = self.v3.rotate_left(21);
        self.v3 ^= self.v0;
        self.v2 = self.v2.wrapping_add(self.v1);
        self.v1 = self.v1.rotate_left(17);
        self.v1 ^= self.v2;
        self.v2 = self.v2.rotate_left(32);
    }

    fn compress(&mut self, m: u64) {
        self.v3 ^= m;
        // SipHash-1-3: 1 compression round
        self.sip_round();
        self.v0 ^= m;
    }
}

impl Hasher for SipHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.length += bytes.len();
        let mut i = 0;

        // Fill the buffer first
        if self.buf_len > 0 {
            while i < bytes.len() && self.buf_len < 8 {
                self.buf[self.buf_len] = bytes[i];
                self.buf_len += 1;
                i += 1;
            }
            if self.buf_len == 8 {
                let m = u64::from_le_bytes(self.buf);
                self.compress(m);
                self.buf_len = 0;
            }
        }

        // Process full 8-byte blocks
        while i + 8 <= bytes.len() {
            let mut block = [0u8; 8];
            block.copy_from_slice(&bytes[i..i + 8]);
            let m = u64::from_le_bytes(block);
            self.compress(m);
            i += 8;
        }

        // Buffer remaining
        while i < bytes.len() {
            self.buf[self.buf_len] = bytes[i];
            self.buf_len += 1;
            i += 1;
        }
    }

    fn finish(&self) -> u64 {
        // Pad the last block
        let mut last = ((self.length as u64) & 0xff) << 56;
        let buf_len = self.buf_len;
        if buf_len >= 7 {
            last |= (self.buf[6] as u64) << 48;
        }
        if buf_len >= 6 {
            last |= (self.buf[5] as u64) << 40;
        }
        if buf_len >= 5 {
            last |= (self.buf[4] as u64) << 32;
        }
        if buf_len >= 4 {
            last |= (self.buf[3] as u64) << 24;
        }
        if buf_len >= 3 {
            last |= (self.buf[2] as u64) << 16;
        }
        if buf_len >= 2 {
            last |= (self.buf[1] as u64) << 8;
        }
        if buf_len >= 1 {
            last |= self.buf[0] as u64;
        }

        let mut v0 = self.v0;
        let mut v1 = self.v1;
        let mut v2 = self.v2;
        let mut v3 = self.v3;

        v3 ^= last;
        // 1 compression round (SipHash-1-3)
        sip_round_vals(&mut v0, &mut v1, &mut v2, &mut v3);
        v0 ^= last;

        // Finalization: 3 rounds
        v2 ^= 0xff;
        sip_round_vals(&mut v0, &mut v1, &mut v2, &mut v3);
        sip_round_vals(&mut v0, &mut v1, &mut v2, &mut v3);
        sip_round_vals(&mut v0, &mut v1, &mut v2, &mut v3);

        v0 ^ v1 ^ v2 ^ v3
    }
}

#[inline]
fn sip_round_vals(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
    *v0 = v0.wrapping_add(*v1);
    *v1 = v1.rotate_left(13);
    *v1 ^= *v0;
    *v0 = v0.rotate_left(32);
    *v2 = v2.wrapping_add(*v3);
    *v3 = v3.rotate_left(16);
    *v3 ^= *v2;
    *v0 = v0.wrapping_add(*v3);
    *v3 = v3.rotate_left(21);
    *v3 ^= *v0;
    *v2 = v2.wrapping_add(*v1);
    *v1 = v1.rotate_left(17);
    *v1 ^= *v2;
    *v2 = v2.rotate_left(32);
}

impl Default for SipHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// A BuildHasher that creates SipHashers (uses fixed keys).
pub struct SipBuildHasher {
    k0: u64,
    k1: u64,
}

impl SipBuildHasher {
    pub const fn new() -> Self {
        Self {
            k0: 0x0706050403020100,
            k1: 0x0f0e0d0c0b0a0908,
        }
    }
}

impl core::hash::BuildHasher for SipBuildHasher {
    type Hasher = SipHasher;
    fn build_hasher(&self) -> SipHasher {
        SipHasher::with_keys(self.k0, self.k1)
    }
}

impl Default for SipBuildHasher {
    fn default() -> Self {
        Self::new()
    }
}
