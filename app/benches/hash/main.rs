/*
 * // Copyright 2024 (c) the Radzivon Bartoshyk. All rights reserved.
 * //
 * // Use of this source code is governed by a BSD-style
 * // license that can be found in the LICENSE file.
 */
use cith::{city_hash128, city_hash256_crc, city_hash32, city_hash64, city_murmur};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.benchmark_group("Hash");

    let bench_test = "MurmurHash1
The original MurmurHash was created as an attempt to make a faster function than Lookup3.[7] Although successful, it had not been tested thoroughly and was not capable of providing 64-bit hashes as in Lookup3. Its design would be later built upon in MurmurHash2, combining a multiplicative hash (similar to the Fowler–Noll–Vo hash function) with an Xorshift.

MurmurHash2
MurmurHash2[8] yields a 32- or 64-bit value. It comes in multiple variants, including some that allow incremental hashing and aligned or neutral versions.

MurmurHash2 (32-bit, x86)—The original version; contains a flaw that weakens collision in some cases.[9]
MurmurHash2A (32-bit, x86)—A fixed variant using Merkle–Damgård construction. Slightly slower.
CMurmurHash2A (32-bit, x86)—MurmurHash2A, but works incrementally.
MurmurHashNeutral2 (32-bit, x86)—Slower, but endian- and alignment-neutral.
MurmurHashAligned2 (32-bit, x86)—Slower, but does aligned reads (safer on some platforms).
MurmurHash64A (64-bit, x64)—The original 64-bit version. Optimized for 64-bit arithmetic.
MurmurHash64B (64-bit, x86)—A 64-bit version optimized for 32-bit platforms. It is not a true 64-bit hash due to insufficient mixing of the stripes.[10]
The person who originally found the flaw[clarification needed] in MurmurHash2 created an unofficial 160-bit version of MurmurHash2 called MurmurHash2_160.[11]

MurmurHash3
The current version, completed April 3, 2011, is MurmurHash3,[12][13] which yields a 32-bit or 128-bit hash value. When using 128-bits, the x86 and x64 versions do not produce the same values, as the algorithms are optimized for their respective platforms. MurmurHash3 was released alongside SMHasher, a hash function test suite.".to_string();

    let very_short_key = "MurmurHash2MurmurHash2";
    let v_short = very_short_key.as_bytes().to_vec();

    let v = bench_test.as_bytes().to_vec();

    c.bench_function("city32", |b| {
        b.iter(|| {
            _ = city_hash32(&v);
        })
    });

    c.bench_function("city32_small", |b| {
        b.iter(|| {
            _ = city_hash32(&v_short);
        })
    });


    c.bench_function("city64", |b| {
        b.iter(|| {
            _ = city_hash64(&v);
        })
    });

    c.bench_function("city64_small", |b| {
        b.iter(|| {
            _ = city_hash64(&v_short);
        })
    });

    c.bench_function("city_murmur", |b| {
        b.iter(|| {
            _ = city_murmur(&v);
        })
    });

    c.bench_function("city_hash128", |b| {
        b.iter(|| {
            _ = city_hash128(&v);
        })
    });

    c.bench_function("city_hash256", |b| {
        b.iter(|| {
            _ = city_hash256_crc(&v);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
