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
use crate::city_hash64;
use crate::generic32::city_hash32;
use std::hash::Hasher;

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

pub struct City64Hasher {
    bytes: Vec<u8>,
}

impl Hasher for City64Hasher {
    fn finish(&self) -> u64 {
        city_hash64(self.bytes.as_slice())
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut copy = bytes.to_vec();
        self.bytes.append(&mut copy);
    }
}

impl Default for City64Hasher {
    fn default() -> Self {
        Self { bytes: vec![0; 0] }
    }
}
