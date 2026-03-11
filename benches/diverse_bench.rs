use std::time::Instant;

use compact_dict::dict::Dict;
use compact_dict::dict::ahash::MojoAHashStrHash;
use std::collections::HashMap as StdHashMap;
use hashbrown::HashMap as BrownHashMap;
use fxhash::FxHashMap;
use indexmap::IndexMap;

fn main() {
    let n = 1_000_000;
    println!("Generating {} keys...", n);
    let all_keys: Vec<String> = (0..n).map(|i| format!("k{}", i)).collect();
    let keys = &all_keys;

    bench_read_heavy(keys, n);
    bench_high_load_factor(keys, n);
    bench_unsuccessful_lookups(keys, n);
}

fn bench_read_heavy(keys: &[String], n: usize) {
    println!("\n============================================================");
    println!("Scenario: Read-Heavy Workload (10% Writes, 90% Reads)");
    println!("N = {}", n);
    println!("============================================================");

    // compact_dict_ahash
    {
        let mut dic: Dict<i32, MojoAHashStrHash> = Dict::new(n);
        let start = Instant::now();
        for key in keys { dic.put(key, 1); }
        let mut sum = 0;
        for _ in 0..9 {
            for key in keys { sum += dic.get_or(key, 0); }
        }
        println!("compact_dict_ahash: {:.6} s (Sum: {})", start.elapsed().as_secs_f64(), sum);
    }

    // std_hashmap
    {
        let mut dic = StdHashMap::with_capacity(n);
        let start = Instant::now();
        for key in keys { dic.insert(key.clone(), 1); }
        let mut sum = 0;
        for _ in 0..9 {
            for key in keys { sum += dic.get(key).unwrap_or(&0); }
        }
        println!("std_hashmap     : {:.6} s (Sum: {})", start.elapsed().as_secs_f64(), sum);
    }

    // hashbrown
    {
        let mut dic = BrownHashMap::with_capacity(n);
        let start = Instant::now();
        for key in keys { dic.insert(key.clone(), 1); }
        let mut sum = 0;
        for _ in 0..9 {
            for key in keys { sum += dic.get(key).unwrap_or(&0); }
        }
        println!("hashbrown       : {:.6} s (Sum: {})", start.elapsed().as_secs_f64(), sum);
    }
}

fn bench_high_load_factor(keys: &[String], n: usize) {
    println!("\n============================================================");
    println!("Scenario: High Load Factor (>85%)");
    println!("N = {}", n);
    println!("============================================================");

    let capacity = (n as f64 / 0.85) as usize; // Force ~85% load factor

    // compact_dict_ahash
    {
        // For compact-dict, passing `capacity` as the initial sizing avoids resizing
        let mut dic: Dict<i32, MojoAHashStrHash> = Dict::new(capacity);
        let start = Instant::now();
        for key in keys { dic.put(key, 1); }
        let mut sum = 0;
        for key in keys { sum += dic.get_or(key, 0); }
        println!("compact_dict_ahash: {:.6} s (Sum: {})", start.elapsed().as_secs_f64(), sum);
    }

    // std_hashmap
    {
        let mut dic = StdHashMap::with_capacity(capacity);
        let start = Instant::now();
        for key in keys { dic.insert(key.clone(), 1); }
        let mut sum = 0;
        for key in keys { sum += dic.get(key).unwrap_or(&0); }
        println!("std_hashmap     : {:.6} s (Sum: {})", start.elapsed().as_secs_f64(), sum);
    }

    // hashbrown
    {
        let mut dic = BrownHashMap::with_capacity(capacity);
        let start = Instant::now();
        for key in keys { dic.insert(key.clone(), 1); }
        let mut sum = 0;
        for key in keys { sum += dic.get(key).unwrap_or(&0); }
        println!("hashbrown       : {:.6} s (Sum: {})", start.elapsed().as_secs_f64(), sum);
    }
}

fn bench_unsuccessful_lookups(keys: &[String], n: usize) {
    println!("\n============================================================");
    println!("Scenario: Unsuccessful Lookups (100% Misses)");
    println!("N = {}", n);
    println!("============================================================");

    let missing_keys: Vec<String> = (n..n*2).map(|i| format!("k{}", i)).collect();

    // compact_dict_ahash
    {
        let mut dic: Dict<i32, MojoAHashStrHash> = Dict::new(n);
        for key in keys { dic.put(key, 1); }
        
        let start = Instant::now();
        let mut missed = 0;
        for key in &missing_keys {
            if dic.get_or(key, 0) == 0 { missed += 1; }
        }
        println!("compact_dict_ahash: {:.6} s (Missed: {})", start.elapsed().as_secs_f64(), missed);
    }

    // std_hashmap
    {
        let mut dic = StdHashMap::with_capacity(n);
        for key in keys { dic.insert(key.clone(), 1); }
        
        let start = Instant::now();
        let mut missed = 0;
        for key in &missing_keys {
            if dic.get(key).is_none() { missed += 1; }
        }
        println!("std_hashmap     : {:.6} s (Missed: {})", start.elapsed().as_secs_f64(), missed);
    }

    // hashbrown
    {
        let mut dic = BrownHashMap::with_capacity(n);
        for key in keys { dic.insert(key.clone(), 1); }
        
        let start = Instant::now();
        let mut missed = 0;
        for key in &missing_keys {
            if dic.get(key).is_none() { missed += 1; }
        }
        println!("hashbrown       : {:.6} s (Missed: {})", start.elapsed().as_secs_f64(), missed);
    }
}
