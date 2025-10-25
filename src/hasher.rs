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
use crate::city32::city_hash32;
use crate::{city_hash64, city_hash64_with_seed};
use std::hash::Hasher;

/// A [`Hasher`] implementation using the 32-bit variant of CityHash.
///
/// `City32Hasher` computes a fast, non-cryptographic 32-bit hash of input data
/// using the [`city_hash32`] algorithm. It is optimized for short and medium-length
/// inputs and provides good distribution for general-purpose hashing tasks.
///
/// This hasher can be used with collections such as [`HashMap`] or [`HashSet`]
/// where a lightweight and deterministic hash function is desired.
pub struct City32Hasher {
    bytes: Vec<u8>,
}

impl Hasher for City32Hasher {
    fn finish(&self) -> u64 {
        city_hash32(self.bytes.as_slice()) as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut copy = bytes.to_vec();
        self.bytes.append(&mut copy);
    }
}

impl Default for City32Hasher {
    fn default() -> Self {
        Self { bytes: vec![0; 0] }
    }
}

/// A [`Hasher`] implementation based on the 64-bit variant of CityHash.
///
/// `City64Hasher` computes a 64-bit non-cryptographic hash of input data using
/// the [`city_hash64`] algorithm. It is designed for speed and good distribution
/// over typical short and medium-length inputs.
///
/// This hasher can be used with collections like [`HashMap`] or [`HashSet`] when
/// deterministic, fast hashing is desired (e.g., for data indexing, caching, or
/// fingerprinting).
pub struct City64Hasher {
    bytes: Vec<u8>,
    seed: u64,
}

impl Hasher for City64Hasher {
    fn finish(&self) -> u64 {
        if self.seed != 0 {
            city_hash64_with_seed(self.bytes.as_slice(), self.seed)
        } else {
            city_hash64(self.bytes.as_slice())
        }
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut copy = bytes.to_vec();
        self.bytes.append(&mut copy);
    }
}

impl Default for City64Hasher {
    fn default() -> Self {
        Self {
            bytes: vec![0; 0],
            seed: 0,
        }
    }
}

impl City64Hasher {
    /// Creates a new [`City64Hasher`] instance with the default seed value (zero).
    ///
    /// This is a convenience constructor equivalent to calling [`Default::default()`].
    /// It initializes an empty hasher using the **CityHash64** algorithm without any seed.
    pub fn new() -> City64Hasher {
        City64Hasher::default()
    }

    /// Creates a new [`City64Hasher`] initialized with a custom 64-bit seed.
    ///
    /// This constructor allows controlling the seed used in the **CityHash64**
    /// algorithm. A unique seed can help generate distinct hash values for the
    /// same input data, reducing collision risks in certain use cases like
    /// multiple hash tables.
    pub fn new_with_seed(seed: u64) -> City64Hasher {
        Self {
            bytes: vec![0; 0],
            seed,
        }
    }
}
