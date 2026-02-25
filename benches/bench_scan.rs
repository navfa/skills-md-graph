use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use skills_md_graph::config::ScanConfig;
use skills_md_graph::parser::{scan_directory, scan_directory_async};

const FILE_COUNT: usize = 10_000;

fn generate_fixtures(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    for i in 0..FILE_COUNT {
        let deps = if i > 0 {
            format!("  - skill-{}", i - 1)
        } else {
            String::new()
        };
        let content = format!(
            "---\nname: skill-{i}\ndescription: Generated skill number {i}\ndependencies:\n{deps}\n---\n\n# Skill {i}\n\nBody content for skill {i}.\n"
        );
        fs::write(dir.join(format!("skill-{i}.md")), content).unwrap();
    }
}

fn bench_sync(dir: &Path) -> std::time::Duration {
    let start = Instant::now();
    let result = scan_directory(dir).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(result.skills.len(), FILE_COUNT);
    elapsed
}

async fn bench_async(dir: &Path, workers: usize) -> std::time::Duration {
    let scan_config = ScanConfig {
        workers,
        extensions: vec!["md".to_string()],
    };
    let aliases = HashMap::new();

    let start = Instant::now();
    let result = scan_directory_async(dir, &scan_config, &aliases, false)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    assert_eq!(result.skills.len(), FILE_COUNT);
    elapsed
}

#[tokio::main]
async fn main() {
    let bench_dir = std::env::temp_dir().join("skill-graph-bench-10k");

    println!("Generating {FILE_COUNT} skill files...");
    generate_fixtures(&bench_dir);
    println!("Done.\n");

    // Sync benchmark
    println!("--- Sync scan ---");
    let sync_duration = bench_sync(&bench_dir);
    println!("  {FILE_COUNT} files in {sync_duration:.2?}");

    // Async benchmarks with varying worker counts
    for workers in [1, 4, 8, 16] {
        println!("\n--- Async scan ({workers} workers) ---");
        let async_duration = bench_async(&bench_dir, workers).await;
        let speedup = sync_duration.as_secs_f64() / async_duration.as_secs_f64();
        println!("  {FILE_COUNT} files in {async_duration:.2?} ({speedup:.2}x vs sync)");
    }

    // Cleanup
    fs::remove_dir_all(&bench_dir).unwrap();
    println!("\nBenchmark complete. Temp files cleaned up.");
}
