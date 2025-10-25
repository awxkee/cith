/*
 * // Copyright (c) Radzivon Bartoshyk 10/2025. All rights reserved.
 * //
 * // Redistribution and use in source and binary forms, with or without modification,
 * // are permitted provided that the following conditions are met:
 * //
 * // 1.  Redistributions of source code must retain the above copyright notice, this
 * // list of conditions and the following disclaimer.
 * //
 * // 2.  Redistributions in binary form must reproduce the above copyright notice,
 * // this list of conditions and the following disclaimer in the documentation
 * // and/or other materials provided with the distribution.
 * //
 * // 3.  Neither the name of the copyright holder nor the names of its
 * // contributors may be used to endorse or promote products derived from
 * // this software without specific prior written permission.
 * //
 * // THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * // AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * // IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * // DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * // FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * // DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * // SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * // CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * // OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * // OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
use crate::city32::permute3;
use crate::city64::{K0, hash_len_16_u64, read_le64, shift_mix};
use std::ops::Not;

#[derive(Copy, Clone, Default)]
pub struct Hash256 {
    pub lo: u128,
    pub hi: u128,
}

#[allow(unused_assignments)]
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "crc")]
fn city256_long_arm_crc(bytes: &[u8], seed: u32) -> Hash256 {
    debug_assert!(bytes.len() >= 240);
    let mut a = read_le64(bytes, 56).wrapping_add(K0);
    let mut b = read_le64(bytes, 96).wrapping_add(K0);
    let mut result0 = hash_len_16_u64(b, bytes.len() as u64);
    let mut c = result0;
    let mut result1 = read_le64(bytes, 120)
        .wrapping_mul(K0)
        .wrapping_add(bytes.len() as u64);
    let mut d = result1;
    let mut e = read_le64(bytes, 184).wrapping_add(seed as u64);
    let mut f = 0u64;
    let mut g = 0u64;
    let mut h = c.wrapping_add(d);
    let mut x = seed as u64;
    let mut y = 0u64;
    let mut z = 0u64;

    let mut len = bytes.len();

    let mut iters = bytes.len() / 240;
    len -= iters * 240;
    #[allow(unused)]
    let mut moved_offset = 0usize;
    use std::arch::aarch64::__crc32cd;
    macro_rules! chunk {
        ($r:expr) => {{
            permute3(&mut x, &mut z, &mut y);
            let chunk = &bytes[moved_offset..moved_offset + 40];
            b = b.wrapping_add(read_le64(chunk, 0));
            c = c.wrapping_add(read_le64(chunk, 8));
            d = d.wrapping_add(read_le64(chunk, 16));
            e = e.wrapping_add(read_le64(chunk, 24));
            f = f.wrapping_add(read_le64(chunk, 32));
            a = a.wrapping_add(b);
            h = h.wrapping_add(f);
            b = b.wrapping_add(c);
            f = f.wrapping_add(d);
            g = g.wrapping_add(e);
            e = e.wrapping_add(z);
            g = g.wrapping_add(x);
            z = __crc32cd(z as u32, b.wrapping_add(g)) as u64;
            y = __crc32cd(y as u32, e.wrapping_add(h)) as u64;
            x = __crc32cd(x as u32, f.wrapping_add(a)) as u64;
            e = e.rotate_right($r);
            c = c.wrapping_add(e);
            moved_offset += 40;
        }};
    }
    loop {
        chunk!(0);
        permute3(&mut a, &mut h, &mut c);
        chunk!(33);
        permute3(&mut a, &mut h, &mut f);
        chunk!(0);
        permute3(&mut b, &mut h, &mut f);
        chunk!(42);
        permute3(&mut b, &mut h, &mut d);
        chunk!(0);
        permute3(&mut b, &mut h, &mut e);
        chunk!(33);
        permute3(&mut a, &mut h, &mut e);
        iters -= 1;
        if iters == 0 {
            break;
        }
    }
    while len >= 40 {
        chunk!(29);
        e ^= a.rotate_right(20);
        h = h.wrapping_add(b.rotate_right(30));
        g ^= c.rotate_right(40);
        f = f.wrapping_add(d.rotate_right(34));
        permute3(&mut c, &mut h, &mut g);
        len -= 40;
    }
    if len > 0 {
        moved_offset = moved_offset + len - 40;
        chunk!(33);
        e ^= a.rotate_right(43);
        h = h.wrapping_add(b.rotate_right(42));
        g ^= c.rotate_right(41);
        f = f.wrapping_add(d.rotate_right(40));
    }
    result0 ^= h;
    result1 ^= g;
    g = g.wrapping_add(h);
    a = hash_len_16_u64(a, g.wrapping_add(z));
    x = x.wrapping_add(y << 32);
    b = b.wrapping_add(x);
    c = hash_len_16_u64(c, z).wrapping_add(h);
    d = hash_len_16_u64(d, e.wrapping_add(result0));
    g = g.wrapping_add(e);
    h = h.wrapping_add(hash_len_16_u64(x, f));
    e = hash_len_16_u64(a, d).wrapping_add(g);
    z = hash_len_16_u64(b, c).wrapping_add(a);
    y = hash_len_16_u64(g, h).wrapping_add(c);
    result0 = e.wrapping_add(z.wrapping_add(y.wrapping_add(x)));
    a = shift_mix(a.wrapping_add(y).wrapping_mul(K0))
        .wrapping_mul(K0)
        .wrapping_add(b);
    result1 = result1.wrapping_add(a.wrapping_add(result0));
    a = shift_mix(a.wrapping_mul(K0))
        .wrapping_mul(K0)
        .wrapping_add(c);
    let result2 = a.wrapping_add(result1);
    a = shift_mix(a.wrapping_add(e).wrapping_mul(K0)).wrapping_mul(K0);
    let result3 = a.wrapping_add(result2);

    let lo = (result0 as u128) | (result1 as u128).wrapping_shl(64);
    let hi = (result2 as u128) | (result3 as u128).wrapping_shl(64);
    Hash256 { lo, hi }
}

#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
fn city256_long_sse42_crc(bytes: &[u8], seed: u32) -> Hash256 {
    debug_assert!(bytes.len() >= 240);
    let mut a = read_le64(bytes, 56).wrapping_add(K0);
    let mut b = read_le64(bytes, 96).wrapping_add(K0);
    let mut result0 = hash_len_16_u64(b, bytes.len() as u64);
    let mut c = result0;
    let mut result1 = read_le64(bytes, 120)
        .wrapping_mul(K0)
        .wrapping_add(bytes.len() as u64);
    let mut d = result1;
    let mut e = read_le64(bytes, 184).wrapping_add(seed as u64);
    let mut f = 0u64;
    let mut g = 0u64;
    let mut h = c.wrapping_add(d);
    let mut x = seed as u64;
    let mut y = 0u64;
    let mut z = 0u64;

    let mut len = bytes.len();

    let mut iters = bytes.len() / 240;
    len -= iters * 240;
    #[allow(unused)]
    let mut moved_offset = 0usize;
    use std::arch::x86_64::_mm_crc32_u64;
    macro_rules! chunk {
        ($r:expr) => {{
            permute3(&mut x, &mut z, &mut y);
            let chunk = &bytes[moved_offset..moved_offset + 40];
            b = b.wrapping_add(read_le64(chunk, 0));
            c = c.wrapping_add(read_le64(chunk, 8));
            d = d.wrapping_add(read_le64(chunk, 16));
            e = e.wrapping_add(read_le64(chunk, 24));
            f = f.wrapping_add(read_le64(chunk, 32));
            a = a.wrapping_add(b);
            h = h.wrapping_add(f);
            b = b.wrapping_add(c);
            f = f.wrapping_add(d);
            g = g.wrapping_add(e);
            e = e.wrapping_add(z);
            g = g.wrapping_add(x);
            z = _mm_crc32_u64(z, b.wrapping_add(g)) as u64;
            y = _mm_crc32_u64(y, e.wrapping_add(h)) as u64;
            x = _mm_crc32_u64(x, f.wrapping_add(a)) as u64;
            e = e.rotate_right($r);
            c = c.wrapping_add(e);
            moved_offset += 40;
        }};
    }
    loop {
        chunk!(0);
        permute3(&mut a, &mut h, &mut c);
        chunk!(33);
        permute3(&mut a, &mut h, &mut f);
        chunk!(0);
        permute3(&mut b, &mut h, &mut f);
        chunk!(42);
        permute3(&mut b, &mut h, &mut d);
        chunk!(0);
        permute3(&mut b, &mut h, &mut e);
        chunk!(33);
        permute3(&mut a, &mut h, &mut e);
        iters -= 1;
        if iters == 0 {
            break;
        }
    }
    while len >= 40 {
        chunk!(29);
        e ^= a.rotate_right(20);
        h = h.wrapping_add(b.rotate_right(30));
        g ^= c.rotate_right(40);
        f = f.wrapping_add(d.rotate_right(34));
        permute3(&mut c, &mut h, &mut g);
        len -= 40;
    }
    if len > 0 {
        moved_offset = moved_offset + len - 40;
        chunk!(33);
        e ^= a.rotate_right(43);
        h = h.wrapping_add(b.rotate_right(42));
        g ^= c.rotate_right(41);
        f = f.wrapping_add(d.rotate_right(40));
    }
    result0 ^= h;
    result1 ^= g;
    g = g.wrapping_add(h);
    a = hash_len_16_u64(a, g.wrapping_add(z));
    x = x.wrapping_add(y << 32);
    b = b.wrapping_add(x);
    c = hash_len_16_u64(c, z).wrapping_add(h);
    d = hash_len_16_u64(d, e.wrapping_add(result0));
    g = g.wrapping_add(e);
    h = h.wrapping_add(hash_len_16_u64(x, f));
    e = hash_len_16_u64(a, d).wrapping_add(g);
    z = hash_len_16_u64(b, c).wrapping_add(a);
    y = hash_len_16_u64(g, h).wrapping_add(c);
    result0 = e.wrapping_add(z.wrapping_add(y.wrapping_add(x)));
    a = shift_mix(a.wrapping_add(y).wrapping_mul(K0))
        .wrapping_mul(K0)
        .wrapping_add(b);
    result1 = result1.wrapping_add(a.wrapping_add(result0));
    a = shift_mix(a.wrapping_mul(K0))
        .wrapping_mul(K0)
        .wrapping_add(c);
    let result2 = a.wrapping_add(result1);
    a = shift_mix(a.wrapping_add(e).wrapping_mul(K0)).wrapping_mul(K0);
    let result3 = a.wrapping_add(result2);

    let lo = (result0 as u128) | (result1 as u128).wrapping_shl(64);
    let hi = (result2 as u128) | (result3 as u128).wrapping_shl(64);
    Hash256 { lo, hi }
}

#[allow(unused_assignments)]
fn city256_long_crc(bytes: &[u8], seed: u32) -> Hash256 {
    debug_assert!(bytes.len() >= 240);
    let mut a = read_le64(bytes, 56).wrapping_add(K0);
    let mut b = read_le64(bytes, 96).wrapping_add(K0);
    let mut result0 = hash_len_16_u64(b, bytes.len() as u64);
    let mut c = result0;
    let mut result1 = read_le64(bytes, 120)
        .wrapping_mul(K0)
        .wrapping_add(bytes.len() as u64);
    let mut d = result1;
    let mut e = read_le64(bytes, 184).wrapping_add(seed as u64);
    let mut f = 0u64;
    let mut g = 0u64;
    let mut h = c.wrapping_add(d);
    let mut x = seed as u64;
    let mut y = 0u64;
    let mut z = 0u64;

    let mut len = bytes.len();

    let mut iters = bytes.len() / 240;
    len -= iters * 240;
    #[allow(unused)]
    let mut moved_offset = 0usize;
    use crate::crc::crc32c_u64;
    macro_rules! chunk {
        ($r:expr) => {{
            permute3(&mut x, &mut z, &mut y);
            let chunk = &bytes[moved_offset..moved_offset + 40];
            b = b.wrapping_add(read_le64(chunk, 0));
            c = c.wrapping_add(read_le64(chunk, 8));
            d = d.wrapping_add(read_le64(chunk, 16));
            e = e.wrapping_add(read_le64(chunk, 24));
            f = f.wrapping_add(read_le64(chunk, 32));
            a = a.wrapping_add(b);
            h = h.wrapping_add(f);
            b = b.wrapping_add(c);
            f = f.wrapping_add(d);
            g = g.wrapping_add(e);
            e = e.wrapping_add(z);
            g = g.wrapping_add(x);
            z = crc32c_u64(z as u32, b.wrapping_add(g)) as u64;
            y = crc32c_u64(y as u32, e.wrapping_add(h)) as u64;
            x = crc32c_u64(x as u32, f.wrapping_add(a)) as u64;
            e = e.rotate_right($r);
            c = c.wrapping_add(e);
            moved_offset += 40;
        }};
    }
    loop {
        chunk!(0);
        permute3(&mut a, &mut h, &mut c);
        chunk!(33);
        permute3(&mut a, &mut h, &mut f);
        chunk!(0);
        permute3(&mut b, &mut h, &mut f);
        chunk!(42);
        permute3(&mut b, &mut h, &mut d);
        chunk!(0);
        permute3(&mut b, &mut h, &mut e);
        chunk!(33);
        permute3(&mut a, &mut h, &mut e);
        iters -= 1;
        if iters == 0 {
            break;
        }
    }
    while len >= 40 {
        chunk!(29);
        e ^= a.rotate_right(20);
        h = h.wrapping_add(b.rotate_right(30));
        g ^= c.rotate_right(40);
        f = f.wrapping_add(d.rotate_right(34));
        permute3(&mut c, &mut h, &mut g);
        len -= 40;
    }
    if len > 0 {
        moved_offset = moved_offset + len - 40;
        chunk!(33);
        e ^= a.rotate_right(43);
        h = h.wrapping_add(b.rotate_right(42));
        g ^= c.rotate_right(41);
        f = f.wrapping_add(d.rotate_right(40));
    }
    result0 ^= h;
    result1 ^= g;
    g = g.wrapping_add(h);
    a = hash_len_16_u64(a, g.wrapping_add(z));
    x = x.wrapping_add(y << 32);
    b = b.wrapping_add(x);
    c = hash_len_16_u64(c, z).wrapping_add(h);
    d = hash_len_16_u64(d, e.wrapping_add(result0));
    g = g.wrapping_add(e);
    h = h.wrapping_add(hash_len_16_u64(x, f));
    e = hash_len_16_u64(a, d).wrapping_add(g);
    z = hash_len_16_u64(b, c).wrapping_add(a);
    y = hash_len_16_u64(g, h).wrapping_add(c);
    result0 = e.wrapping_add(z.wrapping_add(y.wrapping_add(x)));
    a = shift_mix(a.wrapping_add(y).wrapping_mul(K0))
        .wrapping_mul(K0)
        .wrapping_add(b);
    result1 = result1.wrapping_add(a.wrapping_add(result0));
    a = shift_mix(a.wrapping_mul(K0))
        .wrapping_mul(K0)
        .wrapping_add(c);
    let result2 = a.wrapping_add(result1);
    a = shift_mix(a.wrapping_add(e).wrapping_mul(K0)).wrapping_mul(K0);
    let result3 = a.wrapping_add(result2);

    let lo = (result0 as u128) | (result1 as u128).wrapping_shl(64);
    let hi = (result2 as u128) | (result3 as u128).wrapping_shl(64);
    Hash256 { lo, hi }
}

#[inline]
pub(crate) fn city256_long_crc_target(bytes: &[u8], seed: u32) -> Hash256 {
    use std::sync::OnceLock;
    type HashFn = unsafe fn(&[u8], u32) -> Hash256;
    static EXECUTOR: OnceLock<HashFn> = OnceLock::new();

    let func = EXECUTOR.get_or_init(|| {
        #[cfg(target_arch = "x86_64")]
        {
            if std::arch::is_x86_feature_detected!("sse4.2") {
                return city256_long_sse42_crc;
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("crc") {
                return city256_long_arm_crc;
            }
        }

        city256_long_crc
    });
    unsafe { func(bytes, seed) }
}

/// Computes a 256-bit CityHash CRC hash of the given byte slice with a custom seed.
///
/// This function produces a `Hash256` value for the input data using
/// a CRC-based CityHash variant, which is suitable for fast hashing of
/// large datasets with a reasonable level of collision resistance.
///
/// # Parameters
///
/// - `bytes`: The input byte slice to hash.
/// - `seed`: A 32-bit seed value to initialize the hash, allowing for
///   different hash outputs for the same input.
///
/// # Returns
///
/// A `Hash256` containing the 256-bit hash of the input.
pub fn city_hash256_crc_with_seed(bytes: &[u8], seed: u32) -> Hash256 {
    if bytes.len() >= 240 {
        city256_long_crc_target(bytes, seed)
    } else {
        let mut buf = [0u8; 240];
        let len = bytes.len().min(240);
        buf[..len].copy_from_slice(&bytes[..len]);
        city256_long_crc_target(&buf, (len as u32).not())
    }
}

/// Computes a 256-bit CRC-based CityHash of the given byte slice.
///
/// This function produces a `Hash256` value for the input data using
/// a CRC-accelerated variant of CityHash, providing fast hashing
/// suitable for large datasets.
///
/// # Parameters
///
/// - `bytes`: The input byte slice to hash.
///
/// # Returns
///
/// A `Hash256` containing the 256-bit hash of the input.
pub fn city_hash256_crc(bytes: &[u8]) -> Hash256 {
    city_hash256_crc_with_seed(bytes, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_hash_city256_2_with_seed() {
        if std::arch::is_aarch64_feature_detected!("crc") {
            let data2 = b"The current version, completed April 3, 2011, is MurmurHash3,[12][13] which yields a 32-bit or 128-bit hash value. When using 128-bits, the x86 and x64 versions do not produce the same values, as the algorithms are optimized for their respective platforms. MurmurHash3 was released alongside SMHasher, a hash function test suite.";
            let hash2 = unsafe { city256_long_arm_crc(data2, 0) };
            assert_eq!(hash2.lo, 37959015251717061403964514692924197017);
            assert_eq!(hash2.hi, 204623909270555374608389012571575072155);
        }

        if std::arch::is_aarch64_feature_detected!("crc") {
            let data2 = b"CRCs are based on the theory of cyclic error-correcting codes. The use of systematic cyclic codes, which encode messages by adding a fixed-length check value, for the purpose of error detection in communication networks, was first proposed by W. Wesley Peterson in 1961.[2] Cyclic codes are not only simple to implement but have the benefit of being particularly well suited for the detection of burst errors: contiguous sequences of erroneous data symbols in messages. This is important because burst errors are common transmission errors in many communication channels, including magnetic and optical storage devices. Typically an n-bit CRC applied to a data block of arbitrary length will detect any single error burst not longer than n bits, and the fraction of all longer error bursts that it will detect is approximately (1-2n).";
            let hash2 = unsafe { city256_long_arm_crc(data2, 0) };
            assert_eq!(hash2.lo, 159059450530848839484415022192514717329);
            assert_eq!(hash2.hi, 128150177799555610101972982489767506394);
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_hash_city256_2_with_seed() {
        if std::arch::is_x86_feature_detected!("sse4.2") {
            let data2 = b"The current version, completed April 3, 2011, is MurmurHash3,[12][13] which yields a 32-bit or 128-bit hash value. When using 128-bits, the x86 and x64 versions do not produce the same values, as the algorithms are optimized for their respective platforms. MurmurHash3 was released alongside SMHasher, a hash function test suite.";
            let hash2 = unsafe { city256_long_sse42_crc(data2, 0) };
            assert_eq!(hash2.lo, 37959015251717061403964514692924197017);
            assert_eq!(hash2.hi, 204623909270555374608389012571575072155);
        }

        if std::arch::is_x86_feature_detected!("sse4.2") {
            let data2 = b"CRCs are based on the theory of cyclic error-correcting codes. The use of systematic cyclic codes, which encode messages by adding a fixed-length check value, for the purpose of error detection in communication networks, was first proposed by W. Wesley Peterson in 1961.[2] Cyclic codes are not only simple to implement but have the benefit of being particularly well suited for the detection of burst errors: contiguous sequences of erroneous data symbols in messages. This is important because burst errors are common transmission errors in many communication channels, including magnetic and optical storage devices. Typically an n-bit CRC applied to a data block of arbitrary length will detect any single error burst not longer than n bits, and the fraction of all longer error bursts that it will detect is approximately (1-2n).";
            let hash2 = unsafe { city256_long_sse42_crc(data2, 0) };
            assert_eq!(hash2.lo, 159059450530848839484415022192514717329);
            assert_eq!(hash2.hi, 128150177799555610101972982489767506394);
        }
    }

    #[test]
    fn test_hash_city256_2_with_seed_generic() {
        let data2 = b"The current version, completed April 3, 2011, is MurmurHash3,[12][13] which yields a 32-bit or 128-bit hash value. When using 128-bits, the x86 and x64 versions do not produce the same values, as the algorithms are optimized for their respective platforms. MurmurHash3 was released alongside SMHasher, a hash function test suite.";
        let hash2 = city256_long_crc(data2, 0);
        assert_eq!(hash2.lo, 37959015251717061403964514692924197017);
        assert_eq!(hash2.hi, 204623909270555374608389012571575072155);

        let data2 = b"CRCs are based on the theory of cyclic error-correcting codes. The use of systematic cyclic codes, which encode messages by adding a fixed-length check value, for the purpose of error detection in communication networks, was first proposed by W. Wesley Peterson in 1961.[2] Cyclic codes are not only simple to implement but have the benefit of being particularly well suited for the detection of burst errors: contiguous sequences of erroneous data symbols in messages. This is important because burst errors are common transmission errors in many communication channels, including magnetic and optical storage devices. Typically an n-bit CRC applied to a data block of arbitrary length will detect any single error burst not longer than n bits, and the fraction of all longer error bursts that it will detect is approximately (1-2n).";
        let hash2 = city256_long_crc(data2, 0);
        assert_eq!(hash2.lo, 159059450530848839484415022192514717329);
        assert_eq!(hash2.hi, 128150177799555610101972982489767506394);
    }

    #[test]
    fn test_hash_city256_small() {
        let data2 = b"Hello CRC";
        let hash2 = city_hash256_crc_with_seed(data2, 0);
        assert_eq!(hash2.lo, 167610683394798017944502699170498217074);
        assert_eq!(hash2.hi, 250595889336278130356695761762065153882);
    }
}
