use std::time::Instant;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::fs::OpenOptions;

use compact_dict::dict::Dict;
use compact_dict::dict::ahash::MojoAHashStrHash;
use std::collections::HashMap as StdHashMap;
use hashbrown::HashMap as BrownHashMap;
use fxhash::FxHashMap;
use indexmap::IndexMap;

const SIZES: &[usize] = &[1_000, 10_000, 50_000, 100_000, 500_000, 1_000_000, 5_000_000, 10_000_000];

fn main() {
    let max_n = *SIZES.iter().max().unwrap();
    println!("Generating {} keys...", max_n);
    let all_keys: Vec<String> = (0..max_n).map(|i| format!("k{}", i)).collect();

    // Ensure the results directory exists
    std::fs::create_dir_all("results").unwrap();

    for &n in SIZES {
        println!("\n============================================================");
        println!("Benchmarking with N = {} (Dataset strings + values)", n);
        println!("============================================================");
        let keys = &all_keys[0..n];

        // --- compact_dict_ahash ---
        {
            let mut dic: Dict<i32, MojoAHashStrHash> = Dict::new(n);
            let start = Instant::now();
            
            for (i, key) in keys.iter().enumerate() {
                dic.put(key, (i % 7) as i32);
            }

            for (_, key) in keys.iter().enumerate().step_by(2) {
                let val = dic.get_or(key, 0);
                dic.put(key, val * 2);
            }

            let mut sum_val = 0;
            for key in keys {
                sum_val += dic.get_or(key, -1);
            }
            
            let elapsed = start.elapsed().as_secs_f64();
            println!("compact_dict_ahash: {:.6} s (Sum: {})", elapsed, sum_val);
            write_results("compact_dict_ahash", n, elapsed, sum_val);
        }

        // --- fxhash ---
        {
            let mut dic = FxHashMap::default();
            dic.reserve(n); 
            let start = Instant::now();
            
            for (i, key) in keys.iter().enumerate() {
                dic.insert(key.clone(), (i % 7) as i32);
            }

            for (_, key) in keys.iter().enumerate().step_by(2) {
                if let Some(val) = dic.get_mut(key) {
                    *val *= 2;
                } else {
                    dic.insert(key.clone(), 0);
                }
            }

            let mut sum_val = 0;
            for key in keys {
                sum_val += dic.get(key).copied().unwrap_or(-1);
            }
            
            let elapsed = start.elapsed().as_secs_f64();
            println!("fxhash          : {:.6} s (Sum: {})", elapsed, sum_val);
            write_results("fxhash", n, elapsed, sum_val);
        }

        // --- std_hashmap ---
        {
            let mut dic = StdHashMap::with_capacity(n);
            let start = Instant::now();
            
            for (i, key) in keys.iter().enumerate() {
                dic.insert(key.clone(), (i % 7) as i32);
            }

            for (_, key) in keys.iter().enumerate().step_by(2) {
                if let Some(val) = dic.get_mut(key) {
                    *val *= 2;
                } else {
                    dic.insert(key.clone(), 0);
                }
            }

            let mut sum_val = 0;
            for key in keys {
                sum_val += dic.get(key).copied().unwrap_or(-1);
            }
            
            let elapsed = start.elapsed().as_secs_f64();
            println!("std_hashmap     : {:.6} s (Sum: {})", elapsed, sum_val);
            write_results("std_hashmap", n, elapsed, sum_val);
        }

        // --- hashbrown ---
        {
            let mut dic = BrownHashMap::with_capacity(n);
            let start = Instant::now();
            
            for (i, key) in keys.iter().enumerate() {
                dic.insert(key.clone(), (i % 7) as i32);
            }

            for (_, key) in keys.iter().enumerate().step_by(2) {
                if let Some(val) = dic.get_mut(key) {
                    *val *= 2;
                } else {
                    dic.insert(key.clone(), 0);
                }
            }

            let mut sum_val = 0;
            for key in keys {
                sum_val += dic.get(key).copied().unwrap_or(-1);
            }
            
            let elapsed = start.elapsed().as_secs_f64();
            println!("hashbrown       : {:.6} s (Sum: {})", elapsed, sum_val);
            write_results("hashbrown", n, elapsed, sum_val);
        }

        // --- indexmap ---
        {
            let mut dic = IndexMap::with_capacity(n);
            let start = Instant::now();
            
            for (i, key) in keys.iter().enumerate() {
                dic.insert(key.clone(), (i % 7) as i32);
            }

            for (_, key) in keys.iter().enumerate().step_by(2) {
                if let Some(val) = dic.get_mut(key) {
                    *val *= 2;
                } else {
                    dic.insert(key.clone(), 0);
                }
            }

            let mut sum_val = 0;
            for key in keys {
                sum_val += dic.get(key).copied().unwrap_or(-1);
            }
            
            let elapsed = start.elapsed().as_secs_f64();
            println!("indexmap        : {:.6} s (Sum: {})", elapsed, sum_val);
            write_results("indexmap", n, elapsed, sum_val);
        }
    }
}

fn write_results(name: &str, n: usize, elapsed: f64, sum_val: i32) {
    let file_name = format!("{}_{}.json", name, n);
    let results_dir_path = Path::new("results").join(file_name);
    let results = format!(r#"{{"program": "{}", "n": {}, "time_sec": {:.6}, "sum": {}}}"#, name, n, elapsed, sum_val);
    
    let mut file = File::create(results_dir_path).unwrap();
    writeln!(file, "{}", results).unwrap();

    let csv_path = Path::new("results").join(format!("{}.csv", name));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(csv_path)
        .unwrap();

    writeln!(file, "{}, {:.6}", n, elapsed).unwrap();
}
