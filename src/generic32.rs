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

pub fn city_hash32(bytes: &[u8]) -> u32 {
    city_hash32_impl(bytes)
}

const C1: u32 = 0xcc9e2d51;
const C2: u32 = 0x1b873593;

#[inline]
fn fmix(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

#[inline]
fn mur(mut a: u32, mut h: u32) -> u32 {
    // Helper from Murmur3 for combining two 32-bit values.
    a = a.wrapping_mul(C1);
    a = a.rotate_right(17);
    a = a.wrapping_mul(C2);
    h ^= a;
    h = h.rotate_right(19);
    h.wrapping_mul(5).wrapping_add(0xe6546b64)
}

fn hash32_len4(bytes: &[u8]) -> u32 {
    let mut b: u32 = 0u32;
    let mut c = 9u32;
    for &char in bytes.iter() {
        let v: i8 = char as i8;
        b = b.wrapping_mul(C1).wrapping_add(v as u32);
        c ^= b;
    }
    fmix(mur(b, mur(bytes.len() as u32, c)))
}

#[inline]
fn hash32_len5to12(bytes: &[u8]) -> u32 {
    let len = bytes.len();
    let mut a = bytes.len() as u32;
    let mut b = a * 5;
    let mut c = 9;
    let d = b;
    let u0 = &bytes[..4];
    let start1 = len - 4;
    let u1 = &bytes[start1..start1 + 4];
    let start2 = (len >> 1) & 4;
    let u2 = &bytes[start2..start2 + 4];
    a += u32::from_ne_bytes([u0[0], u0[1], u0[2], u0[3]]).to_le();
    b += u32::from_ne_bytes([u1[0], u1[1], u1[2], u1[3]]).to_le();
    c += u32::from_ne_bytes([u2[0], u2[1], u2[2], u2[3]]).to_le();
    fmix(mur(c, mur(b, mur(a, d))))
}

#[inline]
pub(crate) fn read_le32(bytes: &[u8], from_start: usize) -> u32 {
    let u = &bytes[from_start..from_start + 4];
    u32::from_ne_bytes([u[0], u[1], u[2], u[3]]).to_le()
}

#[inline]
fn hash32_len13to24(bytes: &[u8]) -> u32 {
    let len = bytes.len();
    let a = read_le32(bytes, (len >> 1) - 4);
    let b = read_le32(bytes, 4);
    let c = read_le32(bytes, len - 8);
    let d = read_le32(bytes, len >> 1);
    let e = read_le32(bytes, 0);
    let f = read_le32(bytes, len - 4);
    let h = len as u32;

    fmix(mur(f, mur(e, mur(d, mur(c, mur(b, mur(a, h)))))))
}

#[inline]
fn permute3<T>(a: &mut T, b: &mut T, c: &mut T) {
    std::mem::swap(a, b);
    std::mem::swap(a, c);
}

fn city_hash32_impl(bytes: &[u8]) -> u32 {
    if bytes.len() <= 24 {
        let len = bytes.len() as u32;
        if len <= 12 {
            return if len <= 4 {
                hash32_len4(bytes)
            } else {
                hash32_len5to12(bytes)
            };
        }
        return hash32_len13to24(bytes);
    }

    // len > 24
    let ulen = bytes.len();
    let mut h = ulen as u32;
    let mut g = C1.wrapping_mul(h);
    let mut f = g;
    let a0 = read_le32(bytes, ulen - 4)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let a1 = read_le32(bytes, ulen - 8)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let a2 = read_le32(bytes, ulen - 16)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let a3 = read_le32(bytes, ulen - 12)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let a4 = read_le32(bytes, ulen - 20)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    h ^= a0;
    h = h.rotate_right(19);
    h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    h ^= a2;
    h = h.rotate_right(19);
    h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    g ^= a1;
    g = g.rotate_right(19);
    g = g.wrapping_mul(5).wrapping_add(0xe6546b64);
    g ^= a3;
    g = g.rotate_right(19);
    g = g.wrapping_mul(5).wrapping_add(0xe6546b64);
    f = f.wrapping_add(a4);
    f = f.rotate_right(19);
    f = f.wrapping_mul(5).wrapping_add(0xe6546b64);
    let iters = (bytes.len() - 1) / 20;
    for chunk in bytes.chunks_exact(20).take(iters) {
        let a0 = read_le32(chunk, 0)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        let a1 = read_le32(chunk, 4);
        let a2 = read_le32(chunk, 8)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        let a3 = read_le32(chunk, 12)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        let a4 = read_le32(chunk, 16);
        h ^= a0;
        h = h.rotate_right(18);
        h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
        f = f.wrapping_add(a1);
        f = f.rotate_right(19);
        f = f.wrapping_mul(C1);
        g = g.wrapping_add(a2);
        g = g.rotate_right(18);
        g = g.wrapping_mul(5).wrapping_add(0xe6546b64);
        h ^= a3.wrapping_add(a1);
        h = h.rotate_right(19);
        h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
        g ^= a4;
        g = g.swap_bytes().wrapping_mul(5);
        h = h.wrapping_add(a4.wrapping_mul(5));
        h = h.swap_bytes();
        f = f.wrapping_add(a0);
        permute3(&mut f, &mut h, &mut g);
    }
    g = g.rotate_right(11).wrapping_mul(C1);
    g = g.rotate_right(17).wrapping_mul(C1);
    f = f.rotate_right(11).wrapping_mul(C1);
    f = f.rotate_right(17).wrapping_mul(C1);
    h = h.wrapping_add(g).rotate_right(19);
    h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    h = h.rotate_right(17).wrapping_mul(C1);
    h = h.wrapping_add(f).rotate_right(19);
    h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    h = h.rotate_right(17).wrapping_mul(C1);
    h
}

#[cfg(test)]
mod tests {
    use super::city_hash32_impl;

    #[test]
    fn test_empty() {
        let data: &[u8] = &[];
        let hash = city_hash32_impl(data);
        let hs: u32 = cityhasher::hash(data);
        assert_eq!(hash, hs);
    }

    #[test]
    fn test_dog() {
        let data2 = b"The quick brown fox jumps over the lazy dog";
        let hash2 = city_hash32_impl(data2);
        assert_eq!(hash2, 0xa339c810);
    }

    #[test]
    fn test_inputs() {
        let data = [
            b"expected CityHash32 for \"hello\"".to_vec(),
            b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Praesent maximus tincidunt sagittis. Donec a sem laoreet, ullamcorper libero non, suscipit magna. Cras at imperdiet ipsum. Phasellus pharetra porta odio, ac gravida ipsum auctor at. Aliquam.".to_vec(),
            b"Morbi in vulputate sapien. Etiam in convallis lorem. Nullam tempus ipsum aliquam arcu efficitur malesuada. Nunc nibh eros, fringilla eu elit vel, tempus imperdiet dolor. Mauris consequat rutrum neque in vehicula. Integer at dui non eros dapibus tincidunt vitae ac tortor. Vestibulum pretium pharetra sollicitudin. Nam maximus nec ipsum quis vehicula. Phasellus egestas, nibh quis.".to_vec()
        ];
        let expected = [0xa2e9e6d4u32, 0x6466c086u32, 0x4f6ef482u32];
        for (input, &expected) in data.iter().zip(expected.iter()) {
            let hash = city_hash32_impl(input);
            assert_eq!(hash, expected);
        }
    }

    #[test]
    fn test_different_lengths() {
        let short = b"a";
        let long = b"aaaaaaaaaa";
        let hash_short = city_hash32_impl(short);
        let hash_long = city_hash32_impl(long);
        assert_ne!(hash_short, hash_long);
    }

    #[test]
    fn test_consistency() {
        let data = b"repeatable";
        let hash1 = city_hash32_impl(data);
        let hash2 = city_hash32_impl(data);
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_various_inputs() {
        static CONTROL: [&str; 128] = [
            "0xdc56d17a",
            "0xc0a92754",
            "0x2a1678b6",
            "0xd7c992e2",
            "0x616e1132",
            "0xfe6e37d4",
            "0x5145897e",
            "0xcfea845d",
            "0xeb0fd2d6",
            "0x7cd3d6e0",
            "0x14e52250",
            "0x931da128",
            "0xde42ef1c",
            "0x8add7404",
            "0x69976bd0",
            "0xcacd0542",
            "0x17aebf87",
            "0x1e9bcbda",
            "0xba680c4b",
            "0xf7cfbfda",
            "0xc41a2a96",
            "0x4a3f2b87",
            "0x4b7dd7b7",
            "0x1de0e4f5",
            "0x60cf6aa4",
            "0x2e6ddf78",
            "0x17a5df60",
            "0x100139a4",
            "0xef678131",
            "0xc158707d",
            "0xd3d91d57",
            "0xe4345328",
            "0x68943315",
            "0xe14e6d9e",
            "0x33f96086",
            "0x7549c70a",
            "0x43f74d13",
            "0x26911fcc",
            "0xc4c58416",
            "0x0a104234",
            "0x5042df8c",
            "0x7044f7ca",
            "0x055dbbbf",
            "0xce39467e",
            "0xfdb76981",
            "0x716e5ed0",
            "0x05d0c428",
            "0x1950b972",
            "0x99fa0a24",
            "0xf531d568",
            "0x0789e78e",
            "0x56cd4f9b",
            "0x2b0a28b4",
            "0x22498538",
            "0x2691989e",
            "0x82ee00db",
            "0xdc77f8be",
            "0xfeb09805",
            "0x05ad5eea",
            "0xc449c697",
            "0x8ebc55ce",
            "0xc119319b",
            "0x42561cd0",
            "0xefb0d898",
            "0x53d2a4c3",
            "0x7a49977d",
            "0x1a7a3b62",
            "0xc9cbb478",
            "0xd04a9ff8",
            "0x1607f8fa",
            "0x1f71fd8b",
            "0xd44629b3",
            "0x5d709512",
            "0x9cd509c2",
            "0x8db9c50a",
            "0x9497192a",
            "0xa0090ca2",
            "0xbe3f0434",
            "0xfe2c61ea",
            "0x4ea7f018",
            "0xdd027e4a",
            "0x7e8714f9",
            "0x812718c7",
            "0x27319a37",
            "0x1bec91e2",
            "0xd7fa11f6",
            "0x2a4d4a55",
            "0xb456be6f",
            "0xe417cd30",
            "0x5728c0fb",
            "0x5b566ca8",
            "0x7b146f39",
            "0x0c197980",
            "0x28add1ff",
            "0x3e7095b3",
            "0xeb0ab40f",
            "0x611a1bec",
            "0xc60b1d1f",
            "0xfb75ed9f",
            "0xe5c2ed21",
            "0x56e258e5",
            "0xb9b1f97a",
            "0xcbbab8b3",
            "0x1c8bcd5a",
            "0x360ff812",
            "0x593d2794",
            "0xe3b5e900",
            "0x79b282bc",
            "0x91559ebf",
            "0x64af1417",
            "0x42ad606f",
            "0xe9101891",
            "0xaa82f3b0",
            "0xfb4bd972",
            "0xa8c7ae49",
            "0xce7a4050",
            "0x94a56c68",
            "0xf2919a58",
            "0x36cb6074",
            "0x127e2749",
            "0xe00d134e",
            "0x4221cee7",
            "0x573feb8c",
            "0x14df4883",
            "0xdc8c23fb",
            "0xbdbf5cba",
            "0x81dc28d9",
            "0x558331a9",
        ];
        for len in 0..128 {
            let data: Vec<u8> = (0..len).map(|x| x as u8).collect();
            let hash = city_hash32_impl(&data);
            let s = CONTROL[len].trim_start_matches("0x");
            assert_eq!(hash, u32::from_str_radix(s, 16).unwrap());
        }
    }
}
