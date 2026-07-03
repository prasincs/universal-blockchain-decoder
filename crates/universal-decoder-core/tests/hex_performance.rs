//! Performance validation tests for the vendored hex implementation.
//!
//! These tests ensure that the refactored hex implementation using
//! optimized lookup tables performs significantly better than the
//! naive format!-based approach.

use std::time::Instant;
use universal_decoder_core::hex;

/// Wall-clock assertions are meaningless under coverage instrumentation:
/// llvm-cov-instrumented binaries run several times slower, so timing
/// thresholds that comfortably hold in normal builds fail spuriously.
/// cargo-llvm-cov sets LLVM_PROFILE_FILE for the test process; skip the
/// timing ASSERTIONS there (the code under test still runs, so coverage
/// is unaffected - only the threshold check is bypassed).
fn skip_timing_assertions() -> bool {
    let under_coverage = std::env::var_os("LLVM_PROFILE_FILE").is_some();
    if under_coverage {
        println!("coverage instrumentation detected - timing assertions skipped");
    }
    under_coverage
}

/// Test that verifies the hex encoding doesn't have obvious performance issues.
/// This is a smoke test, not a full benchmark.
#[test]
fn test_encode_performance_reasonable() {
    // Create test data (10KB)
    let data: Vec<u8> = (0..10_000).map(|i| (i % 256) as u8).collect();

    // Warm up
    for _ in 0..10 {
        let _ = hex::encode(&data);
    }

    // Time 100 iterations
    let start = Instant::now();
    for _ in 0..100 {
        let _ = hex::encode(&data);
    }
    let elapsed = start.elapsed();

    // With optimized implementation, 100 iterations of 10KB encoding
    // should complete in well under 100ms on any reasonable hardware.
    // The old format!-based implementation would take ~500ms+.
    println!("Encoded 10KB 100 times in {:?}", elapsed);
    if skip_timing_assertions() {
        return;
    }
    assert!(
        elapsed.as_millis() < 100,
        "Encoding took {:?}, expected < 100ms (likely using slow format! implementation)",
        elapsed
    );
}

/// Test that verifies hex decoding performance is reasonable.
#[test]
fn test_decode_performance_reasonable() {
    // Create test data (10KB of hex = 20KB string)
    let data: Vec<u8> = (0..10_000).map(|i| (i % 256) as u8).collect();
    let hex_data = hex::encode(&data);

    // Warm up
    for _ in 0..10 {
        let _ = hex::decode(&hex_data).unwrap();
    }

    // Time 100 iterations
    let start = Instant::now();
    for _ in 0..100 {
        let _ = hex::decode(&hex_data).unwrap();
    }
    let elapsed = start.elapsed();

    // With optimized implementation, 100 iterations of 20KB decoding
    // should complete in well under 200ms. Decoding is more complex than
    // encoding (involves error checking and bit operations per byte).
    println!("Decoded 20KB hex 100 times in {:?}", elapsed);
    if skip_timing_assertions() {
        return;
    }
    assert!(
        elapsed.as_millis() < 200,
        "Decoding took {:?}, expected < 200ms",
        elapsed
    );
}

/// Test that the optimized implementation produces correct results
/// even under stress (large data).
#[test]
fn test_large_data_correctness() {
    // 1MB of data
    let data: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();

    let encoded = hex::encode(&data);
    assert_eq!(encoded.len(), 2_000_000); // 2 hex chars per byte

    let decoded = hex::decode(&encoded).expect("Failed to decode");
    assert_eq!(decoded, data);
}

/// Verify that the iterator-based encoding doesn't have unexpected
/// allocation patterns by testing with various sizes.
#[test]
fn test_encoding_scales_linearly() {
    let sizes = [100, 1_000, 10_000, 100_000];
    let mut times = Vec::new();

    for &size in &sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

        let start = Instant::now();
        for _ in 0..100 {
            let _ = hex::encode(&data);
        }
        let elapsed = start.elapsed();
        times.push(elapsed);

        println!("Size {}: {:?}", size, elapsed);
    }

    // Verify rough linear scaling (allowing for some variance)
    // 10x more data should take roughly 5-15x more time (not 50x+)
    let ratio_10x = times[1].as_nanos() as f64 / times[0].as_nanos() as f64;
    let ratio_100x = times[2].as_nanos() as f64 / times[0].as_nanos() as f64;

    println!("10x scaling ratio: {:.2}x", ratio_10x);
    println!("100x scaling ratio: {:.2}x", ratio_100x);

    // Should scale linearly, not quadratically
    assert!(
        ratio_10x < 20.0,
        "10x data took {:.2}x longer (expected ~10x, got worse than quadratic)",
        ratio_10x
    );
    assert!(
        ratio_100x < 200.0,
        "100x data took {:.2}x longer (expected ~100x, got worse than quadratic)",
        ratio_100x
    );
}

/// Test decode_to_slice performance (new functionality not in old wrapper).
#[test]
fn test_decode_to_slice_performance() {
    let hex_data = "48656c6c6f20776f726c6421".repeat(1000); // ~24KB
    let mut buffer = vec![0u8; hex_data.len() / 2];

    // Warm up
    for _ in 0..10 {
        hex::decode_to_slice(&hex_data, &mut buffer).unwrap();
    }

    // Time 100 iterations
    let start = Instant::now();
    for _ in 0..100 {
        hex::decode_to_slice(&hex_data, &mut buffer).unwrap();
    }
    let elapsed = start.elapsed();

    println!("decode_to_slice 100 times in {:?}", elapsed);
    if skip_timing_assertions() {
        return;
    }
    assert!(
        elapsed.as_millis() < 100,
        "decode_to_slice took {:?}, expected < 100ms",
        elapsed
    );
}
