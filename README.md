# compact-dict

`compact-dict` is an experimental, highly customizable, open-addressing dictionary in Rust. It features linear probing and continuous memory layout for string keys, heavily inspired by Mojo's Dict.

## Features

- **Customizable Layout:** Generic integer types for key-count (`KC`) and key-offset (`KO`) sizes. This flexibility allows you to perfectly balance memory usage (e.g. `u16` vs `u32` indices) depending on your scale requirements.
- **Cache Friendly:** Linear probing design with optional cached hashes (`CACHING_HASHES`).
- **SIMD Preparation:** Includes preparations for SIMD-accelerated probing (via `portable_simd`).
- **Configurable Mutability:** Optional destructive operations via the `DESTRUCTIVE` constant generic parameter, which toggles bitsets to mask deleted operations cleanly payload.
- **Performant Hash Function:** By default, it uses high-performance internal hasher variants built upon aHash (like `MojoAHashStrHash`).

## Requirements

- **Rust Nightly**: The crate strictly uses `#![feature(portable_simd)]` and relies on the `std::simd` features, meaning you must compile it with the nightly Rust toolchain.

## Usage

Below is a standard usage example:

```rust
use compact_dict::dict::Dict;

fn main() {
    // Creating a new Dict: Values are of type `u32`
    // Internally uses default Configs: MojoAHashStrHash, u32 for indices
    let mut map: Dict<u32> = Dict::new(8);

    // Insert elements
    map.put("hello", 42);
    map.put("world", 100);

    // Retrieval
    assert_eq!(map.get_or("hello", 0), 42);
    assert_eq!(map.get_or("world", 0), 100);

    // Missing key returns default
    assert_eq!(map.get_or("missing", 0), 0);

    // Metadata checks
    assert_eq!(map.contains("hello"), true);
    assert_eq!(map.len(), 2);
    
    // Reset the map
    map.clear();
    assert_eq!(map.len(), 0);
}
```

## Running Tests

To test the package, run cargo test utilizing the nightly toolchain. 

```bash
cargo +nightly test
```

Benchmarking:

```bash
RUSTFLAGS="-C target-cpu=native" cargo +nightly bench --bench workload_bench
```

Results:

compact_dict_fx: 0.127 s 🚀🔥
fxhash: 0.181 s
hashbrown: 0.188 s
std_hashmap: 0.238 s