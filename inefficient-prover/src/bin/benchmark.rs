use bls12_381_prover::*;
use ark_ff::UniformRand;
use ark_std::test_rng;
use std::time::Instant;

#[derive(Debug)]
struct BenchmarkResult {
    log_n: usize,
    elements: usize,
    setup_time: f64,
    prover_time: u128,
    throughput: f64,
    verify_time: u128,
}

fn main() {
    println!("BLS12-381 Prover Performance Benchmark");
    println!("======================================\n");
    
    // Test different sizes: 2^10, 2^12, 2^14, 2^16
    let test_sizes = vec![10, 12, 14, 16];
    let mut results = Vec::new();
    
    for log_n in test_sizes {
        let n = 1 << log_n;
        println!("Benchmarking n = 2^{} ({} elements)...", log_n, n);
        
        let config = Config { log_n };
        
        // Setup phase
        let setup_start = Instant::now();
        let setup = Setup::new(config);
        let setup_time = setup_start.elapsed();
        
        // Prover phase
        let prover_start = Instant::now();
        let prover = Prover::new(setup.clone());
        let (commitment, polynomial_evals) = prover.prove();
        let prover_time = prover_start.elapsed();
        
        // Opening proof
        let mut rng = test_rng();
        let eval_point = Fr::rand(&mut rng);
        let opening_proof = prover.create_opening_proof(&polynomial_evals, eval_point);
        
        // Verification
        let verify_start = Instant::now();
        let verifier = Verifier::new(setup);
        let is_valid = verifier.verify_opening(&commitment, &opening_proof);
        let verify_time = verify_start.elapsed();
        
        // Calculate throughput
        let elements_per_sec = n as f64 / prover_time.as_secs_f64();
        
        // Store result
        results.push(BenchmarkResult {
            log_n,
            elements: n,
            setup_time: setup_time.as_secs_f64(),
            prover_time: prover_time.as_millis(),
            throughput: elements_per_sec,
            verify_time: verify_time.as_millis(),
        });
        
        // Verify correctness
        assert!(is_valid, "Verification failed for n=2^{}", log_n);
        println!("✓ Completed n = 2^{}\n", log_n);
    }
    
    // Print complete results table
    println!("Benchmark Results:");
    println!("| Size | Elements | Setup Time | Prover Time | Throughput | Verification |");
    println!("|------|----------|------------|-------------|------------|--------------|");
    
    for result in results {
        println!("| n=2^{} | {} | {:.1}s | {}ms | {:.0} elem/s | ~{}ms |",
            result.log_n,
            result.elements,
            result.setup_time,
            result.prover_time,
            result.throughput,
            result.verify_time
        );
    }
    
    println!("\n✓ All benchmarks completed successfully");
} 