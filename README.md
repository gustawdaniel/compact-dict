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

```
compact_dict_fx: 0.166 s 🚀🔥
hashbrown: 0.185 s
fxhash: 0.185 s
std_hashmap: 0.251 s
```

*Note: For the most accurate comparisons without allocation overhead skewing metrics, ensure to run tests natively with `LTO` enabled, as seen in the bench profile.*

## Performance Analysis & Honest Comparison

`compact-dict` is exceptionally fast for **very specific workloads**, beating out highly optimized SwissTable implementations like `hashbrown` and standard `std::collections::HashMap`. However, it makes severe trade-offs to achieve this speed. Here is an honest guide on when to use what:

### ✅ Where `compact-dict` Wins (Strengths)
1. **String-Heavy Initialization & Iteration**: Instead of individually heap-allocating every `String` key like standard HashMaps do, `compact-dict` copies all strings into a single densely-packed, continuous `Vec<u8>` memory buffer (`KeysContainer`). 
2. **SIMD Vectorization**: Lookups leverage `#![feature(portable_simd)]` to compare up to 16 cached `u32` hashes in exactly a single hardware instruction cycle.
3. **Data Analysis & Short-lived Workloads**: If you need to ingest millions of unique strings rapidly, perform some math/updates on their values, and then drop the map, `compact-dict` will significantly outpace its competitors.

### ❌ Where `compact-dict` Loses (Weaknesses / Use `hashbrown` instead)
1. **Continuous Server Deletions**: Deleting elements in `compact-dict` only marks a bit field as deleted (tombstoning). It **never** compacts or frees the physical string memory from the keys buffer. If you implement a long-running web server that constantly adds and removes strings, `compact-dict` will act as a memory leak until the entire dictionary is dropped. 
2. **Generic Keys**: The dictionary is hardcoded around `KeysContainer` offsets, heavily specializing in `&str`. You cannot drop in `HashMap<Uuid, usize>` or custom structs as keys easily. Standard map implementations are completely generic.
3. **Ecosystem Stability**: It relies on Nightly Rust explicitly for `std::simd`. `hashbrown` has zero unstable dependencies and runs practically everywhere perfectly optimized for all target architectures.