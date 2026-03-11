# compact-dict

[![Crates.io](https://img.shields.io/crates/v/compact-dict.svg)](https://crates.io/crates/compact-dict)

`compact-dict` is a highly customizable, open-addressing dictionary in Rust. It features linear probing and continuous memory layout for string keys, heavily inspired by Mojo's Dict.

## Why `compact-dict`?

Modern CPUs hate pointer chasing. While traditional maps often fragment data, `compact-dict` ensures your data stays together.

### Memory Layout vs Traditional Maps

```mermaid
graph TD
    subgraph Separate_Chaining [Traditional Map: Separate Chaining]
        NodeA[Bucket 1] -->|Pointer| NodeB[Heap Node]
        NodeB -->|Pointer| NodeC[Heap Node]
        style NodeB fill:#f96,stroke:#333
        style NodeC fill:#f96,stroke:#333
    end

    subgraph Compact_Dict [compact-dict: Continuous Layout]
        direction LR
        Slot1[K1:V1] --- Slot2[K2:V2] --- Slot3[K3:V3] --- Slot4[K4:V4]
        style Slot1 fill:#4CAF50,stroke:#333
        style Slot2 fill:#4CAF50,stroke:#333
        style Slot3 fill:#4CAF50,stroke:#333
        style Slot4 fill:#4CAF50,stroke:#333
    end

    Note1[Pointer Chasing = Cache Misses] -.-> Separate_Chaining
    Note2[Strict Contiguity = Prefetcher Heaven] -.-> Compact_Dict
```

### Where we fit in

```mermaid
quadrantChart
    title Hash Map Ecosystem Positioning
    x-axis Low Memory Overhead --> High Performance
    y-axis Complex Architecture --> Simple & Local
    quadrant-1 High Speed & Cache Locality
    quadrant-2 Pure Speed Complex SIMD
    quadrant-3 Small Footprint
    quadrant-4 General Purpose
    "std::HashMap (SipHash)": [0.35, 0.4]
    "hashbrown (AHash)": [0.65, 0.35]
    "Separate Chaining Maps": [0.2, 0.8]
    "compact-dict": [0.85, 0.9]
```

*Wait, `std::collections::HashMap` is literally `hashbrown` under the hood! Why does the pure `hashbrown` crate perform better in benchmarks?*
Because the standard library version uses a cryptographically secure hashing algorithm (`SipHash` / `RandomState`) by default to prevent DOS attacks. The raw `hashbrown` crate (and our benchmarks) typically defaults to `AHash` or allows swapping to faster, non-cryptographic hashers. Architecturally, they are the same SwissTable.

## Features

- **Zero-Copy Deserialization (`rkyv`):** Supports `rkyv` for blazing fast zero-copy lookups and memory-mapped files. Enable via the `rkyv` feature!
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

### Zero-Copy Memory Mapping (`rkyv`)

If you enable the `rkyv` feature, you can serialize and memory-map your dictionaries for instant access without any deserialization overhead (saving both time and RAM on massive structures).

```toml
[dependencies]
compact-dict = { version = "0.1.1", features = ["rkyv"] }
```

```rust
use compact_dict::dict::Dict;
use rkyv::ser::{serializers::AllocSerializer, Serializer};

// 1. Serialize
let mut dict = Dict::<u32>::new(16);
dict.put("zero-copy", 42);
let mut serializer = AllocSerializer::<256>::default();
serializer.serialize_value(&dict).unwrap();
let bytes = serializer.into_serializer().into_inner();

// 2. Instant Zero-Copy Lookup (e.g., mapped directly from a file via `mmap`)
// By utilizing relative pointer offsets, the `ArchivedDict` requires absolutely no allocations or deserialization CPU cycles to read.
let archived = unsafe { rkyv::archived_root::<Dict<u32>>(&bytes) };
assert_eq!(archived.get("zero-copy"), Some(42));
```

## Running Tests

To test the package, run cargo test utilizing the nightly toolchain. 

```bash
cargo +nightly test
```

Benchmarking:

There are two benchmarks in the repository: `workload_bench` (simulating real-world string ingestion and mutations) and `dict_bench` (using Criterion for pure primitive operations like `get`).

```bash
RUSTFLAGS="-C target-cpu=native" cargo +nightly bench --bench workload_bench
```

**Workload Benchmark Results (`workload_bench` - Insertions + Mutations):**
```
compact_dict_fx: 0.152 s 🚀🔥
fxhash: 0.182 s
std_hashmap: 0.267 s
hashbrown: 0.272 s
```

**Pure Lookup Results (`dict_bench` - 10k random `get` operations):**

```bash
RUSTFLAGS="-C target-cpu=native" cargo +nightly bench --bench dict_bench
```

```
hashbrown: ~74 µs 🚀🔥
compact_dict: ~143 µs
```

**Scaling Benchmark (`workload_bench` by size):**

To demonstrate the exact boundaries of Cache Locality vs SIMD pointer-chasing, we benchmarked the time to initialize, insert, and query varying dataset sizes.

| Dataset Size | Keys/Values Memory | compact-dict (AHash) | hashbrown | IndexMap | Winner |
| --- | --- | --- | --- | --- | --- |
| **1k** | ~25 KB | 0.00006 s | 0.00005 s | 0.00005 s | **Tie** |
| **10k** | ~250 KB | 0.00056 s | 0.00051 s | 0.00052 s | **Tie** |
| **50k** | ~1.25 MB | 0.00332 s | 0.00294 s | 0.00387 s | **hashbrown (~12% faster)** |
| **100k** | ~2.5 MB | 0.00670 s | 0.00744 s | 0.00716 s | **compact-dict (~10% faster)** |
| **500k** | ~12.5 MB | 0.06847 s | 0.10188 s | 0.10495 s | **compact-dict (~1.5x faster)** 🚀 |
| **1M** | ~25 MB | 0.13251 s | 0.24084 s | 0.24170 s | **compact-dict (~1.8x faster)** 🚀 |
| **5M** | ~120 MB | 1.03868 s | 0.88833 s | 1.45756 s | **hashbrown (~16% faster)** |
| **10M** | ~250 MB | 2.23751 s | 1.92807 s | 3.17355 s | **hashbrown (~16% faster)** |
| **20M** | ~500 MB | 5.90096 s | 6.52020 s | 8.73994 s | **compact-dict (~10% faster)** 🚀 |
| **50M** | ~1.25 GB | 13.56649 s | 19.67217 s | 27.40097 s | **compact-dict (~45% faster)** 🚀 |

**Conclusion**: The scaling behavior reveals an incredibly fascinating bimodal performance curve.
1. **L1/L2 Cache Zone (< 2 MB):** `hashbrown` competes fiercely through raw SIMD optimizations.
2. **L3 Cache Bound (2MB - 50MB):** `compact-dict` utterly destroys pointing-chasing architectures, being nearly 2x faster due to perfect L3 prefetching.
3. **DRAM Latency Zone (50MB - 300MB):** Once the working dataset firmly spills into RAM (e.g. 10M), the continuous memory structure suffers from linear DRAM lookups, allowing `hashbrown`'s highly localized SIMD metadata scans to gracefully overtake it.
4. **Memory Bandwidth Saturation (> 500MB):** At ultra-large scales (50 million elements), the massive pointer graph and memory bloat of traditional maps causes severe cache thrashing array-wide. `compact-dict` **re-takes the lead by a massive 45% margin**, simply because linear array iteration maximizes extreme DRAM memory bandwidth throughput, beating out random pointer lookups.

### Diverse Benchmark Scenarios (1M Elements)

To explore the extreme edges of linear probing, we benchmarked specific scenarios using `diverse_bench`:

| Scenario | compact-dict (AHash) | hashbrown | Winner |
| --- | --- | --- | --- |
| **Read-Heavy Workload (90% Reads)** | 0.324 s | 0.629 s | **compact-dict (~1.9x faster)** 🚀 |
| **High Load Factor (~85%)** | 0.073 s | 0.198 s | **compact-dict (~2.7x faster)** 🚀 |
| **Unsuccessful Lookups (100% Misses)**| 0.021 s | 0.011 s | **hashbrown (~1.9x faster)** |

**Takeaway:**
* `compact-dict` dominates read-heavy sequential workloads perfectly. 
* While high load factors theoretically degrade linear probing, the L3 Cache locality still powers through linear sequences faster than pointer-chasing at 1M scale.
* **The absolute worst-case scenario for `compact-dict` is Unsuccessful Lookups.** Linear probing is forced to search sequentially until it hits an empty slot (which takes longer at higher load factors), making it roughly ~2x slower than `hashbrown`'s SIMD metadata scanning which immediately rejects misses.

*Note: For the most accurate comparisons without allocation overhead skewing metrics, ensure to run tests natively with `LTO` enabled, as seen in the bench profile.*

## ⚖️ Design Trade-offs & Philosophy

`compact-dict` isn't a drop-in replacement for every use case. It is a specialized tool built with specific constraints to achieve maximum throughput.

### 1. The "No-Deletion" Strategy
Currently, `compact-dict` is optimized for **Append-Only** or **Static** workloads. 
* **Why?** Implementing deletions in a linear probing map usually requires either "Tombstones" (which pollute the cache and slow down lookups) or "Backward Shift Deletion" (which is expensive).
* **Status:** If you need frequent `remove()` operations, stick to `hashbrown`. If you need raw lookup speed for datasets that are built once and read many times, this is for you.

### 2. Linear Probing vs. SwissTables (SIMD)
While `hashbrown` uses SIMD instructions to scan metadata buckets, `compact-dict` bets on the modern CPU's **L1/L2 cache prefetcher**.
* **The Bet:** For small to medium-sized maps, the overhead of setting up SIMD registers can be higher than just letting the CPU scan a contiguous block of memory. We prioritize **minimal pointer chasing**.

### 3. Memory Brutalism
We use `std::ptr` and raw memory layouts to bypass some of the overhead of high-level abstractions.
* **Safety:** The core logic is wrapped in `unsafe` blocks where performance dictates it. While we strive for correctness, the primary goal is squeezing every nanosecond out of the hardware.
* **Audit:** We welcome contributors to run `cargo miri test` and help us refine the memory boundaries.

### 4. Load Factor & Clustering
Because we use **Linear Probing**, this map is sensitive to the load factor. 
* To maintain peak performance, we recommend keeping the load factor below **0.7**. 
* Past this point, "Primary Clustering" can occur. We trade this risk for the benefit of extreme cache locality during successful lookups.

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
4. **Pure Lookup Speed**: In pure read-heavy workloads (e.g., retrieving 10,000 strings without inserting or scanning new ones), highly optimized SwissTables like `hashbrown` still outperform `compact-dict`. As seen in the pure-`get` microbenchmark, `hashbrown` can be about **2x faster** than `compact-dict` for isolated random-access lookups. The performance strength of `compact-dict` revolves around the combined speed of contiguous string ingestion, sequential iteration cache-locality, and mutation operations.