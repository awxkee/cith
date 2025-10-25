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
use crate::city64::{K1, hash64_len_0_to_16, read_le64, shift_mix};

#[inline]
fn hash_len16(lo: u64, hi: u64) -> u64 {
    const K_MUL: u64 = 0x9ddfea08eb382d69u64;
    let mut a = (lo ^ hi).wrapping_mul(K_MUL);
    a ^= a >> 47;
    let mut b = (hi ^ a).wrapping_mul(K_MUL);
    b ^= b >> 47;
    b = b.wrapping_mul(K_MUL);
    b
}

#[inline]
fn city_murmur_with_seed_impl(bytes: &[u8], seed: u128) -> u128 {
    let mut a = (seed & 0xffff_ffff_ffff_ffff) as u64;
    let mut b = (seed >> 64) as u64;
    let mut c;
    let mut d;
    let len = bytes.len();
    if len <= 16 {
        a = shift_mix(a.wrapping_mul(K1)).wrapping_mul(K1);
        c = b.wrapping_mul(K1).wrapping_add(hash64_len_0_to_16(bytes));
        d = shift_mix(a.wrapping_add(if len >= 8 { read_le64(bytes, 0) } else { c }));
    } else {
        c = hash_len16(read_le64(bytes, len - 8).wrapping_add(K1), a);
        d = hash_len16(
            b.wrapping_add(len as u64),
            c.wrapping_add(read_le64(bytes, len - 16)),
        );
        a += d;
        let iters = (bytes.len() - 1) / 16;
        let sliced = &bytes[..iters * 16];
        for chunk in sliced.chunks_exact(32) {
            a ^= shift_mix(read_le64(chunk, 0).wrapping_mul(K1)).wrapping_mul(K1);
            a = a.wrapping_mul(K1);
            b ^= a;
            c ^= shift_mix(read_le64(chunk, 8).wrapping_mul(K1)).wrapping_mul(K1);
            c = c.wrapping_mul(K1);
            d ^= c;

            a ^= shift_mix(read_le64(chunk, 16).wrapping_mul(K1)).wrapping_mul(K1);
            a = a.wrapping_mul(K1);
            b ^= a;
            c ^= shift_mix(read_le64(chunk, 24).wrapping_mul(K1)).wrapping_mul(K1);
            c = c.wrapping_mul(K1);
            d ^= c;
        }

        let rem = sliced.chunks_exact(32).remainder();

        for chunk in rem.chunks_exact(16) {
            a ^= shift_mix(read_le64(chunk, 0).wrapping_mul(K1)).wrapping_mul(K1);
            a = a.wrapping_mul(K1);
            b ^= a;
            c ^= shift_mix(read_le64(chunk, 8).wrapping_mul(K1)).wrapping_mul(K1);
            c = c.wrapping_mul(K1);
            d ^= c;
        }
    }
    a = hash_len16(a, c);
    b = hash_len16(d, b);
    ((a ^ b) as u128) | (hash_len16(b, a) as u128).wrapping_shl(64)
}

/// Computes the 128-bit CityMurmur hash of a byte slice using a custom seed.
///
/// CityMurmur is a hybrid non-cryptographic hash function combining
/// ideas from CityHash and MurmurHash. This variant allows specifying
/// a 128-bit seed to alter the output, useful for independent hash
/// streams or avoiding collisions.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to hash.
/// * `seed`  - A 128-bit seed (`u128`) to influence the hash result.
///
/// # Returns
///
/// A `u128` value representing the hash of the input slice combined with the seed.
pub fn city_murmur_with_seed(bytes: &[u8], seed: u128) -> u128 {
    city_murmur_with_seed_impl(bytes, seed)
}

/// Computes the 128-bit CityMurmur hash of a byte slice.
///
/// CityMurmur is a hybrid hash function combining ideas from
/// CityHash and MurmurHash. It is designed for non-cryptographic
/// hashing, producing a 128-bit hash suitable for fingerprinting,
/// checksums, and hash tables.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to hash.
///
/// # Returns
///
/// A `u128` value representing the hash of the input slice.
pub fn city_murmur(bytes: &[u8]) -> u128 {
    city_murmur_with_seed_impl(bytes, 0)
}

#[cfg(test)]
mod tests {
    use crate::murmur::city_murmur;

    #[test]
    fn test_city_murmur() {
        assert_eq!(
            city_murmur(b"123456789"),
            137555568363236656549789161648540888277
        );
        assert_eq!(city_murmur(b"123"), 147516989038733154062198668001837519169);
        assert_eq!(city_murmur(&[]), 236886107234819556091512130834823996519);
        assert_eq!(city_murmur(b"1"), 232547357270412657736765068303298868200);
        assert_eq!(
            city_murmur(b"123456789123456"),
            99170281196658490251255024019638099994
        );
        assert_eq!(
            city_murmur(b"The quick brown fox jumps over the lazy dog"),
            251933285825128086863441730578249075428
        );
        assert_eq!(
            city_murmur(b"The current version, completed April 3, 2011, is MurmurHash3,[12][13] which yields a 32-bit or 128-bit hash value. When using 128-bits, the x86 and x64 versions do not produce the same values, as the algorithms are optimized for their respective platforms. MurmurHash3 was released alongside SMHasher, a hash function test suite."),
            49367195754802758346567295079188952398
        );
    }
}
