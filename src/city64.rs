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
use crate::city32::read_le32;

#[inline]
pub(crate) fn read_le64(bytes: &[u8], from_start: usize) -> u64 {
    let u = &bytes[from_start..from_start + 8];
    u64::from_ne_bytes([u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7]]).to_le()
}

// Some primes between 2^63 and 2^64 for various uses.
pub(crate) const K0: u64 = 0xc3a5c85c97cb3127;
pub(crate) const K1: u64 = 0xb492b66fbe98f273;
pub(crate) const K2: u64 = 0x9ae16a3b2f90404f;

#[inline]
pub(crate) fn hash_len_16_u64(u: u64, v: u64) -> u64 {
    const MUL: u64 = 0x9ddfea08eb382d69;
    hash_len_16_with_mul(u, v, MUL)
}

#[inline]
pub(crate) fn hash_len_16_with_mul(u: u64, v: u64, mul: u64) -> u64 {
    // Murmur-inspired hashing.
    let mut a = (u ^ v).wrapping_mul(mul);
    a ^= a >> 47;
    let mut b = (v ^ a).wrapping_mul(mul);
    b ^= b >> 47;
    b.wrapping_mul(mul)
}

#[inline]
pub(crate) fn shift_mix(val: u64) -> u64 {
    val ^ (val >> 47)
}

#[inline]
pub(crate) fn hash64_len_0_to_16(bytes: &[u8]) -> u64 {
    if bytes.len() >= 8 {
        let mul = K2.wrapping_add((bytes.len() as u64).wrapping_mul(2));
        let a = read_le64(bytes, 0).wrapping_add(K2);
        let b = read_le64(bytes, bytes.len() - 8);
        let c = b.rotate_right(37).wrapping_mul(mul).wrapping_add(a);
        let d = a.rotate_right(25).wrapping_add(b).wrapping_mul(mul);
        hash_len_16_with_mul(c, d, mul)
    } else if bytes.len() >= 4 {
        let mul = K2.wrapping_add((bytes.len() as u64).wrapping_mul(2));
        let a = read_le32(bytes, 0) as u64;
        hash_len_16_with_mul(
            (bytes.len() as u64).wrapping_add(a << 3),
            read_le32(bytes, bytes.len() - 4) as u64,
            mul,
        )
    } else if bytes.len() > 0 {
        let a = bytes[0] as u32;
        let b = bytes[bytes.len() >> 1] as u32;
        let c = bytes[bytes.len() - 1] as u32;
        let y = a.wrapping_add(b << 8);
        let z = (bytes.len() as u32).wrapping_add(c << 2);
        shift_mix((y as u64).wrapping_mul(K2) ^ (z as u64).wrapping_mul(K0)).wrapping_mul(K2)
    } else {
        K2
    }
}

#[inline]
pub(crate) fn hash64_len_17_to_32(bytes: &[u8]) -> u64 {
    let mul = K2.wrapping_add(bytes.len() as u64 * 2);
    let a = read_le64(bytes, 0).wrapping_mul(K1);
    let b = read_le64(bytes, 8);
    let c = read_le64(bytes, bytes.len() - 8).wrapping_mul(mul);
    let d = read_le64(bytes, bytes.len() - 16).wrapping_mul(K2);
    hash_len_16_with_mul(
        a.wrapping_add(b)
            .rotate_right(43)
            .wrapping_add(c.rotate_right(30))
            .wrapping_add(d),
        a.wrapping_add(b.wrapping_add(K2).rotate_right(18))
            .wrapping_add(c),
        mul,
    )
}

#[inline]
fn weak_hash_len_32_with_seeds_impl(w: u64, x: u64, y: u64, z: u64, a: u64, b: u64) -> (u64, u64) {
    let a = a.wrapping_add(w);
    let b = b.wrapping_add(a).wrapping_add(z).rotate_right(21);
    let c = a;
    let a = a.wrapping_add(x);
    let a = a.wrapping_add(y);
    let b = b.wrapping_add(a.rotate_right(44));
    (a.wrapping_add(z), b.wrapping_add(c))
}

#[inline]
pub(crate) fn weak_hash_len_32_with_seeds(
    bytes: &[u8],
    offset: usize,
    a: u64,
    b: u64,
) -> (u64, u64) {
    weak_hash_len_32_with_seeds_impl(
        read_le64(bytes, offset),
        read_le64(bytes, offset + 8),
        read_le64(bytes, offset + 16),
        read_le64(bytes, offset + 24),
        a,
        b,
    )
}

#[inline]
pub(crate) fn hash64_len_33_to_64(bytes: &[u8]) -> u64 {
    let mul = K2.wrapping_add(bytes.len() as u64 * 2);
    let a = read_le64(bytes, 0).wrapping_mul(K2);
    let b = read_le64(bytes, 8);
    let c = read_le64(bytes, bytes.len() - 24);
    let d = read_le64(bytes, bytes.len() - 32);
    let e = read_le64(bytes, 16).wrapping_mul(K2);
    let f = read_le64(bytes, 24).wrapping_mul(9);
    let g = read_le64(bytes, bytes.len() - 8);
    let h = read_le64(bytes, bytes.len() - 16).wrapping_mul(mul);
    let u = a
        .wrapping_add(g)
        .rotate_right(43)
        .wrapping_add(b.rotate_right(30).wrapping_add(c).wrapping_mul(9));
    let v = (a.wrapping_add(g) ^ d).wrapping_add(f).wrapping_add(1);
    let w = u
        .wrapping_add(v)
        .wrapping_mul(mul)
        .swap_bytes()
        .wrapping_add(h);
    let x = e.wrapping_add(f).rotate_right(42).wrapping_add(c);
    let y = v
        .wrapping_add(w)
        .wrapping_mul(mul)
        .swap_bytes()
        .wrapping_add(g)
        .wrapping_mul(mul);
    let z = e.wrapping_add(f).wrapping_add(c);
    let a = x
        .wrapping_add(z)
        .wrapping_mul(mul)
        .wrapping_add(y)
        .swap_bytes()
        .wrapping_add(b);
    let b = shift_mix(
        z.wrapping_add(a)
            .wrapping_mul(mul)
            .wrapping_add(d)
            .wrapping_add(h),
    )
    .wrapping_mul(mul);
    b.wrapping_add(x)
}

#[inline]
fn city_hash64_with_seeds(bytes: &[u8], seed0: u64, seed1: u64) -> u64 {
    hash_len_16_u64(city_hash64(bytes).wrapping_sub(seed0), seed1)
}

/// Computes a 64-bit CityHash value for the given byte slice, using the specified seed.
///
/// This function implements the **CityHash64** algorithm with an additional 64-bit seed.
/// CityHash64 is a fast, non-cryptographic hash function optimized for speed and good
/// bit mixing on short and medium-length inputs.
///
/// Providing a custom seed allows for hash diversification, which can help reduce
/// collision risks when multiple hash tables or hashing contexts share similar data patterns.
///
/// # Parameters
/// - `bytes`: The input data to hash.
/// - `seed`: A 64-bit seed value used to randomize the hash output.
///
/// # Returns
/// A 64-bit hash value computed from `bytes` and `seed`.
pub fn city_hash64_with_seed(bytes: &[u8], seed: u64) -> u64 {
    city_hash64_with_seeds(bytes, K0, seed)
}

/// Computes the 64-bit CityHash of a byte slice.
///
/// CityHash64 is a non-cryptographic hash function optimized for
/// fast hashing of strings and byte arrays. It produces a 64-bit hash
/// value suitable for hash tables, fingerprinting, and checksums.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to hash.
///
/// # Returns
///
/// A `u64` value representing the hash of the input slice.
pub fn city_hash64(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    if len <= 32 {
        if len <= 16 {
            return hash64_len_0_to_16(bytes);
        } else {
            return hash64_len_17_to_32(bytes);
        }
    } else if len <= 64 {
        return hash64_len_33_to_64(bytes);
    }

    // For strings over 64 bytes we hash the end first, and then as we
    // loop we keep 56 bytes of state: v, w, x, y, and z.
    // For strings over 64 bytes we hash the end first, and then as we
    // loop we keep 56 bytes of state: v, w, x, y, and z.
    let mut x = read_le64(bytes, len - 40);
    let mut y = read_le64(bytes, len - 16).wrapping_add(read_le64(bytes, len - 56));
    let mut z = hash_len_16_u64(
        read_le64(bytes, len - 48).wrapping_add(len as u64),
        read_le64(bytes, len - 24),
    );
    let mut v = weak_hash_len_32_with_seeds(bytes, len - 64, len as u64, z);
    let mut w = weak_hash_len_32_with_seeds(bytes, len - 32, y.wrapping_add(K1), x);
    x = x.wrapping_mul(K1).wrapping_add(read_le64(bytes, 0));

    // Decrease len to the nearest multiple of 64, and operate on 64-byte chunks.
    for chunk in bytes[..len - 1].chunks_exact(64) {
        x = x
            .wrapping_add(y)
            .wrapping_add(v.0)
            .wrapping_add(read_le64(chunk, 8))
            .rotate_right(37)
            .wrapping_mul(K1);
        y = y
            .wrapping_add(v.1)
            .wrapping_add(read_le64(chunk, 48))
            .rotate_right(42)
            .wrapping_mul(K1);
        x ^= w.1;
        y = y.wrapping_add(v.0.wrapping_add(read_le64(chunk, 40)));
        z = z.wrapping_add(w.0).rotate_right(33).wrapping_mul(K1);
        v = weak_hash_len_32_with_seeds(chunk, 0, v.1.wrapping_mul(K1), x.wrapping_add(w.0));
        w = weak_hash_len_32_with_seeds(
            chunk,
            32,
            z.wrapping_add(w.1),
            y.wrapping_add(read_le64(chunk, 16)),
        );
        std::mem::swap(&mut z, &mut x);
    }
    hash_len_16_u64(
        hash_len_16_u64(v.0, w.0).wrapping_add(shift_mix(y).wrapping_mul(K1).wrapping_add(z)),
        hash_len_16_u64(v.1, w.1).wrapping_add(x),
    )
}

#[cfg(test)]
mod tests {
    use super::city_hash64;

    #[test]
    fn test_empty() {
        let data: &[u8] = &[];
        let hash = city_hash64(data);
        assert_eq!(hash, 11160318154034397263);
    }

    #[test]
    fn test_dog() {
        let data2 = b"The quick brown fox jumps over the lazy dog";
        let hash2 = city_hash64(data2);
        assert_eq!(hash2, 14008572299481893501);
    }

    #[test]
    fn test_inputs() {
        let data = [
            b"expected CityHash32 for \"hello\"".to_vec(),
            b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Praesent maximus tincidunt sagittis. Donec a sem laoreet, ullamcorper libero non, suscipit magna. Cras at imperdiet ipsum. Phasellus pharetra porta odio, ac gravida ipsum auctor at. Aliquam.".to_vec(),
            b"Morbi in vulputate sapien. Etiam in convallis lorem. Nullam tempus ipsum aliquam arcu efficitur malesuada. Nunc nibh eros, fringilla eu elit vel, tempus imperdiet dolor. Mauris consequat rutrum neque in vehicula. Integer at dui non eros dapibus tincidunt vitae ac tortor. Vestibulum pretium pharetra sollicitudin. Nam maximus nec ipsum quis vehicula. Phasellus egestas, nibh quis.".to_vec()
        ];
        let expected = [
            0x07dfdde413e89ab6u64,
            0xdbd105742c03ccbau64,
            0x128a4f507a1b9834u64,
        ];
        for (input, &expected) in data.iter().zip(expected.iter()) {
            let hash = city_hash64(input);
            assert_eq!(hash, expected);
        }
    }

    #[test]
    fn test_different_lengths() {
        let short = b"a";
        let long = b"aaaaaaaaaa";
        let hash_short = city_hash64(short);
        let hash_long = city_hash64(long);
        assert_ne!(hash_short, hash_long);
    }

    #[test]
    fn test_consistency() {
        let data = b"repeatable";
        let hash1 = city_hash64(data);
        let hash2 = city_hash64(data);
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_various_inputs() {
        static CONTROL: [&str; 128] = [
            "9ae16a3b2f90404f",
            "be6056edf5e94b54",
            "c2a04665ed038d75",
            "94a13d22e9eba49a",
            "82bffd898958e540",
            "b4bfa9e87732c149",
            "92fdbcd8e94a2333",
            "a2e0bff20db0a6a1",
            "ad5a13e1e8e93b98",
            "81371e150e4ad84f",
            "9b704db2b2d6ffea",
            "f3212b3c1d803add",
            "9fd5df33aefc3d7d",
            "c1c5cf5c853d3c92",
            "4bd9a87bb2bb4671",
            "862a51555943bd9d",
            "0efd25a0a34156d4",
            "bbb6a6f8f20d1f1c",
            "79f8c18b091f57e6",
            "9397ab7e5511df31",
            "8dd7e7f1bf16a0e9",
            "ebed9d4fcec9fa24",
            "28e6d435e458f020",
            "0662e334e298fddc",
            "3f3b313dcbd16ec7",
            "b49fc64083cc4c3e",
            "1313dbdeed4e9ded",
            "427e4dbedaea19f6",
            "a56007058a23efa6",
            "96081fedc82adf9f",
            "48abb0e9ebd50ea7",
            "fbd950af27ef6941",
            "1a9d8199972cdf49",
            "46e1378cbc22daba",
            "b5bac3abe70d1522",
            "28e59da3f3008300",
            "0011dfc4eea43101",
            "3645772b95bd3743",
            "629946fa201ac819",
            "a5f6faba3c6fb303",
            "36f7dd7a65e93d1c",
            "c07403048a1eeda7",
            "ca4d3c89744f43a3",
            "4cf58d31ed02513e",
            "9e07c00bd59d7a01",
            "5e776542af3c0519",
            "fa62d2a4bc5d87c6",
            "b7f64877a70ee14a",
            "c7099eac443f4625",
            "60b11c741d047a11",
            "73286092ee9394c9",
            "7877a8d9b5a4b929",
            "d589cb2ce84723a8",
            "a85c240f3e025587",
            "c3d98ecdb6bf52cd",
            "f890c6323948a021",
            "96a5683630a4333f",
            "a84168edcd8646cc",
            "67f1e6f0ccfdf70e",
            "84cb84c6edffeab6",
            "e954fc8f7968db69",
            "b29212c2d2ef6b03",
            "2f20733b20979be3",
            "af30927a77ada6ef",
            "e99ab80f5ec7dca5",
            "ac589c990483dd2e",
            "af6f3ea7de248f72",
            "15452a0335f8d3ff",
            "87faaf9ffc00b792",
            "d735fd9242f41d1d",
            "bebcb91fd6057b44",
            "0177474a5d1ebbe6",
            "9273be0076008b09",
            "4ff1e9fb068a6a2b",
            "a9bd698beab91622",
            "112ef9fd43f6ab0c",
            "bd41cfd5fb432baf",
            "20959f28fef778b5",
            "7dc5c0f6bffa9f3a",
            "5d8d82ca24381650",
            "4feedc7dada54816",
            "993dad25f4602235",
            "48ac17a39fa67d56",
            "a69f17df1e2b88c4",
            "62e9f87e7484d48e",
            "86b1f8e0d03141a2",
            "4ccf1f0ffc2468c4",
            "c0828aa177219532",
            "ffdce2c7bf331b8f",
            "6992c7d1be0fd9ca",
            "d1715e9954348633",
            "cd69d2919e30aa60",
            "a5f2c86dc3e7480b",
            "536f357a66399901",
            "b9da9a2379c19379",
            "364ae6782f889be4",
            "e3f6cd656b9c26be",
            "b1541a33562869ea",
            "72bf686a93a755af",
            "af958d2eb9ed0dff",
            "a47fe83e60b34cc6",
            "d61f11279fb8a7e5",
            "7ebe4b3d74386431",
            "41cc5ad24988a733",
            "0b87a4fb288a06c2",
            "e3c3810bf671386c",
            "7c1f3a08a8e24551",
            "8edc6a63e8b10115",
            "a0ce313ca06c2d01",
            "531f781e2c8264bc",
            "d2a65241d3709bbe",
            "96042182140074fd",
            "c2bb46ba9af19ed3",
            "e031e70d24a3eb33",
            "f9d935c7400b2407",
            "663845cc33b07f0a",
            "ca3fc22c3bcd0200",
            "2c8dc0bcac69bd81",
            "3e49372b20c3da04",
            "bb9384101d3bbe76",
            "612b5e5a1c296054",
            "1f5f351cc71b79a3",
            "5775c3be1a6af9fa",
            "de6a78ea8ba8dd0e",
            "90c6012b151e7de3",
            "8db4fd55a93183f1",
            "b4e37477c19e1ef3",
            "efd614390a7b1d95",
        ];
        for len in 0..128 {
            let data: Vec<u8> = (0..len).map(|x| x as u8).collect();
            let hash = city_hash64(&data);
            let s = CONTROL[len].trim_start_matches("0x");
            assert_eq!(hash, u64::from_str_radix(s, 16).unwrap());
        }
    }
}
