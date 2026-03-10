use std::time::Instant;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::fs::OpenOptions;

use compact_dict::dict::Dict;
use compact_dict::dict::ahash::FxStrHash;
use std::collections::HashMap as StdHashMap;
use hashbrown::HashMap as BrownHashMap;
use fxhash::FxHashMap;

const N: usize = 1_000_000;

fn main() {
    let keys: Vec<String> = (0..N).map(|i| format!("k{}", i)).collect();

    // Ensure the results directory exists
    std::fs::create_dir_all("results").unwrap();

    // --- compact_dict_fx ---
    {
        println!("Running benchmark for compact_dict_fx...");
        let mut dic: Dict<i32, FxStrHash> = Dict::new(N);
        let start = Instant::now();
        
        for (i, key) in keys.iter().enumerate() {
            dic.put(key, (i % 7) as i32);
        }

        for (_, key) in keys.iter().enumerate().step_by(2) {
            let val = dic.get_or(key, 0);
            dic.put(key, val * 2);
        }

        let mut sum_val = 0;
        for key in &keys {
            sum_val += dic.get_or(key, -1);
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        println!("compact_dict_fx: Sum: {}, elapsed sec: {:.6}\n", sum_val, elapsed);

        write_results("compact_dict_fx", elapsed, sum_val);
    }

    // --- std_hashmap ---
    {
        println!("Running benchmark for std_hashmap...");
        let mut dic = StdHashMap::with_capacity(N);
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
        for key in &keys {
            sum_val += dic.get(key).copied().unwrap_or(-1);
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        println!("std_hashmap: Sum: {}, elapsed sec: {:.6}\n", sum_val, elapsed);

        write_results("std_hashmap", elapsed, sum_val);
    }

    // --- hashbrown ---
    {
        println!("Running benchmark for hashbrown...");
        let mut dic = BrownHashMap::with_capacity(N);
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
        for key in &keys {
            sum_val += dic.get(key).copied().unwrap_or(-1);
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        println!("hashbrown: Sum: {}, elapsed sec: {:.6}\n", sum_val, elapsed);

        write_results("hashbrown", elapsed, sum_val);
    }

    // --- fxhash ---
    {
        println!("Running benchmark for fxhash...");
        let mut dic = FxHashMap::default();
        dic.reserve(N); // FxHashMap default doesn't let us pass capacity directly on initialization sometimes, but reserve works
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
        for key in &keys {
            sum_val += dic.get(key).copied().unwrap_or(-1);
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        println!("fxhash: Sum: {}, elapsed sec: {:.6}\n", sum_val, elapsed);

        write_results("fxhash", elapsed, sum_val);
    }
}

fn write_results(name: &str, elapsed: f64, sum_val: i32) {
    let file_name = format!("{}.json", name);
    let results_dir_path = Path::new("results").join(file_name);
    let results = format!(r#"{{"program": "{}", "time_sec": {:.6}, "sum": {}}}"#, name, elapsed, sum_val);
    
    let mut file = File::create(results_dir_path).unwrap();
    writeln!(file, "{}", results).unwrap();

    let csv_path = Path::new("results").join(format!("{}.csv", name));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(csv_path)
        .unwrap();

    writeln!(file, "{:.6}", elapsed).unwrap();
}
