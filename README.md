CityHash

A CityHash implementation in pure Rust.

```rust
use cith::City64Hasher;
use std::hash::Hasher;

fn main() {
    let mut hasher = City64Hasher::new_with_seed(42);

    let data = b"Hash me!";
    hasher.write(data);

    let hash_value = hasher.finish();
    println!("Hash: {}", hash_value);
}
```

----

This project is licensed under either of

- BSD-3-Clause License (see [LICENSE](LICENSE.md))
- Apache License, Version 2.0 (see [LICENSE](LICENSE-APACHE.md))

at your option.
