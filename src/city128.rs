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
use crate::city64::{K0, K1, hash_len_16_u64, read_le64, weak_hash_len_32_with_seeds};
use crate::{city_hash256_crc, city_murmur_with_seed};

#[derive(Copy, Clone, Default)]
struct DeinterleavedU128 {
    lo: u64,
    hi: u64,
}

#[inline]
fn city_128_with_seed_impl(bytes: &[u8], seed: u128) -> u128 {
    if bytes.len() < 128 {
        return city_murmur_with_seed(bytes, seed);
    }
    // We expect len >= 128 to be the common case.  Keep 56 bytes of state:
    // v, w, x, y, and z.
    let mut len = bytes.len() as u64;
    let mut x = (seed & 0xffff_ffff_ffff_ffff) as u64;
    let mut y = (seed >> 64) as u64;
    let mut z = len.wrapping_mul(K1);
    let mut v: DeinterleavedU128 = DeinterleavedU128::default();
    let mut w: DeinterleavedU128 = DeinterleavedU128::default();
    v.lo = (y ^ K1)
        .rotate_right(49)
        .wrapping_mul(K1)
        .wrapping_add(read_le64(bytes, 0));
    v.hi =
        v.lo.rotate_right(42)
            .wrapping_mul(K1)
            .wrapping_add(read_le64(bytes, 8));
    w.lo = y
        .wrapping_add(z)
        .rotate_right(35)
        .wrapping_mul(K1)
        .wrapping_add(x);
    w.hi = x
        .wrapping_add(read_le64(bytes, 88))
        .rotate_right(53)
        .wrapping_mul(K1);

    // This is the same inner loop as CityHash64(), manually unrolled.
    let mut moved_offset = 0usize;
    loop {
        let s = &bytes[moved_offset..moved_offset + 64];
        x = x
            .wrapping_add(y)
            .wrapping_add(v.lo)
            .wrapping_add(read_le64(s, 8))
            .rotate_right(37)
            .wrapping_mul(K1);
        y = y
            .wrapping_add(v.hi.wrapping_add(read_le64(s, 48)))
            .rotate_right(42)
            .wrapping_mul(K1);
        x ^= w.hi;
        y = y.wrapping_add(v.lo.wrapping_add(read_le64(s, 40)));
        z = z.wrapping_add(w.lo).rotate_right(33).wrapping_mul(K1);
        let q0 = weak_hash_len_32_with_seeds(s, 0, v.hi.wrapping_mul(K1), x.wrapping_add(w.lo));
        v = DeinterleavedU128 { lo: q0.0, hi: q0.1 };
        let q1 = weak_hash_len_32_with_seeds(
            s,
            32,
            z.wrapping_add(w.hi),
            y.wrapping_add(read_le64(s, 16)),
        );
        w = DeinterleavedU128 { lo: q1.0, hi: q1.1 };
        std::mem::swap(&mut z, &mut x);
        moved_offset += 64;
        let s = &bytes[moved_offset..moved_offset + 64];
        x = x
            .wrapping_add(y)
            .wrapping_add(v.lo)
            .wrapping_add(read_le64(s, 8))
            .rotate_right(37)
            .wrapping_mul(K1);
        y = y
            .wrapping_add(v.hi.wrapping_add(read_le64(s, 48)))
            .rotate_right(42)
            .wrapping_mul(K1);
        x ^= w.hi;
        y = y.wrapping_add(v.lo.wrapping_add(read_le64(s, 40)));
        z = z.wrapping_add(w.lo).rotate_right(33).wrapping_mul(K1);
        let q0 = weak_hash_len_32_with_seeds(s, 0, v.hi.wrapping_mul(K1), x.wrapping_add(w.lo));
        v = DeinterleavedU128 { lo: q0.0, hi: q0.1 };
        let q1 = weak_hash_len_32_with_seeds(
            s,
            32,
            z.wrapping_add(w.hi),
            y.wrapping_add(read_le64(s, 16)),
        );
        w = DeinterleavedU128 { lo: q1.0, hi: q1.1 };
        std::mem::swap(&mut z, &mut x);
        moved_offset += 64;
        len -= 128;
        if len < 128 {
            break;
        }
    }
    x = x.wrapping_add(v.lo.wrapping_add(z).rotate_right(49).wrapping_mul(K0));
    y = y.wrapping_mul(K0).wrapping_add(w.hi.rotate_right(37));
    z = z.wrapping_mul(K0).wrapping_add(w.lo.rotate_right(27));
    w.lo = w.lo.wrapping_mul(9);
    v.lo = v.lo.wrapping_mul(K0);
    // If 0 < len < 128, hash up to 4 chunks of 32 bytes each from the end of s.
    let mut tail_done = 0;
    while tail_done < len {
        tail_done += 32;
        y = x
            .wrapping_add(y)
            .rotate_right(42)
            .wrapping_mul(K0)
            .wrapping_add(v.hi);
        w.lo = w.lo.wrapping_add(read_le64(
            bytes,
            moved_offset + len as usize - tail_done as usize + 16,
        ));
        x = x.wrapping_mul(K0).wrapping_add(w.lo);
        z = z.wrapping_add(w.hi.wrapping_add(read_le64(
            bytes,
            moved_offset + len as usize - tail_done as usize,
        )));
        w.hi = w.hi.wrapping_add(v.lo);
        let q0 = weak_hash_len_32_with_seeds(
            bytes,
            moved_offset + len as usize - tail_done as usize,
            v.lo.wrapping_add(z),
            v.hi,
        );
        v = DeinterleavedU128 { lo: q0.0, hi: q0.1 };
        v.lo = v.lo.wrapping_mul(K0);
    }
    // At this point our 56 bytes of state should contain more than
    // enough information for a strong 128-bit hash.  We use two
    // different 56-byte-to-8-byte hashes to get a 16-byte final result.
    x = hash_len_16_u64(x, v.lo);
    y = hash_len_16_u64(y.wrapping_add(z), w.lo);
    let z0 = hash_len_16_u64(x.wrapping_add(v.hi), w.hi).wrapping_add(y);
    let z1 = hash_len_16_u64(x.wrapping_add(w.hi), y.wrapping_add(v.hi));
    (z0 as u128) | (z1 as u128).wrapping_shl(64)
}

/// Computes the 128-bit CityHash value of the given byte slice using the provided 128-bit seed.
///
/// CityHash128 is designed for high performance on modern CPUs and produces high-quality,
/// well-distributed 128-bit hashes. It is typically used when a strong hash with low collision
/// probability is required for large inputs.
///
/// This seeded variant allows deterministic diversification of hash outputs, which is useful when
/// combining hashes from different data sources or avoiding predictable results.
///
/// # Parameters
/// - `bytes`: The input data to hash.
/// - `seed`: A 128-bit seed value used to initialize the hash state. Different seeds yield different results.
///
/// # Returns
/// A 128-bit (`u128`) hash value representing the content of `bytes`.
pub fn city_hash128_with_seed(bytes: &[u8], seed: u128) -> u128 {
    city_128_with_seed_impl(bytes, seed)
}

/// Computes a 128-bit CityHash value for the given byte slice.
///
/// CityHash128 produces a fast, high-quality 128-bit hash optimized for modern CPUs.
/// It provides excellent avalanche behavior and uniform distribution, making it suitable
/// for non-cryptographic use cases such as hash tables, checksums, or content fingerprinting.
///
/// Unlike [`city_hash128_with_seed`], this variant uses internally chosen constants as the
/// seed values and produces a deterministic hash for the same input.
///
/// # Parameters
/// - `bytes`: The input data to hash.
///
/// # Returns
/// A 128-bit (`u128`) hash value representing the contents of `bytes`.
pub fn city_hash128(bytes: &[u8]) -> u128 {
    if bytes.len() >= 16 {
        let q0 = read_le64(bytes, 0);
        let q1 = read_le64(bytes, 8).wrapping_add(K0);
        city_128_with_seed_impl(&bytes[16..], (q0 as u128) | (q1 as u128).wrapping_shl(64))
    } else {
        city_128_with_seed_impl(bytes, K0 as u128 | ((K1 as u128).wrapping_shl(64)))
    }
}

/// Computes a 128-bit CRC-based CityHash of the given byte slice.
///
/// This function produces a `u128` hash for the input data using
/// a CRC-accelerated variant of CityHash, which is optimized
/// for performance on CPUs with CRC instructions.
///
/// # Parameters
///
/// - `bytes`: The input byte slice to hash.
///
/// # Returns
///
/// A `u128` value representing the 128-bit hash of the input.
pub fn city_hash128_crc(bytes: &[u8]) -> u128 {
    if bytes.len() <= 900 {
        city_hash128(bytes)
    } else {
        let hash = city_hash256_crc(bytes);
        hash.hi
    }
}

/// Computes a 128-bit CRC-based CityHash of the given byte slice with a seed.
///
/// This function produces a `u128` hash for the input data using
/// a CRC-accelerated variant of CityHash, allowing an additional
/// `seed` to influence the result. The seed is useful for randomized
/// hashing or creating independent hash streams.
///
/// # Parameters
///
/// - `bytes`: The input byte slice to hash.
/// - `seed`: A 128-bit seed value (`u128`) to mix into the hash.
///
/// # Returns
///
/// A `u128` value representing the 128-bit hash of the input, influenced by the seed.
pub fn city_hash128_crc_with_seed(bytes: &[u8], seed: u128) -> u128 {
    if bytes.len() <= 900 {
        city_hash128_with_seed(bytes, seed)
    } else {
        let hash = city_hash256_crc(bytes);
        let result_lo = (hash.lo & 0xffff_ffff_ffff_ffff) as u64;
        let result_hi = (hash.lo >> 64) as u64;

        let result1_lo = (hash.hi & 0xffff_ffff_ffff_ffff) as u64;
        let result1_hi = (hash.hi >> 64) as u64;

        let u = ((seed >> 64) as u64).wrapping_add(result_lo);
        let v = ((seed & 0xffff_ffff_ffff_ffff) as u64).wrapping_add(result_hi);

        let l0 = hash_len_16_u64(u, v.wrapping_add(result1_lo));
        let l1 = hash_len_16_u64(
            v.rotate_right(32),
            u.wrapping_mul(K0).wrapping_add(result1_hi),
        );
        (l0 as u128) | (l1 as u128).wrapping_shl(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_city128() {
        let data2 = b"The original MurmurHash was created as an attempt to make a faster function than Lookup3.[7] Although successful, it had not been tested thoroughly and was not capable of providing 64-bit hashes as in Lookup3. Its design would be later built upon in MurmurHash2, combining a multiplicative hash (similar to the FowlerNollVo hash function) with an Xorshift.";
        let hash2 = city_128_with_seed_impl(data2, 0);
        assert_eq!(hash2, 181738720256903589065179743458014556635);
    }

    #[test]
    fn test_hash_city128_2() {
        let data2 = b"The current version, completed April 3, 2011, is MurmurHash3,[12][13] which yields a 32-bit or 128-bit hash value. When using 128-bits, the x86 and x64 versions do not produce the same values, as the algorithms are optimized for their respective platforms. MurmurHash3 was released alongside SMHasher, a hash function test suite.";
        let hash2 = city_128_with_seed_impl(data2, 0);
        assert_eq!(hash2, 141227953010020849533055099834667401374);
    }

    #[test]
    fn test_hash_city128_2_with_seed() {
        let data2 = b"The current version, completed April 3, 2011, is MurmurHash3,[12][13] which yields a 32-bit or 128-bit hash value. When using 128-bits, the x86 and x64 versions do not produce the same values, as the algorithms are optimized for their respective platforms. MurmurHash3 was released alongside SMHasher, a hash function test suite.";
        let hash2 = city_128_with_seed_impl(data2, 125);
        assert_eq!(hash2, 119575411414761893753960023769141284138);
    }
}
