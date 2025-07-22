use bls12_381_prover::*;
use ark_ff::{UniformRand};
use ark_std::test_rng;

fn main() {
    println!("BLS12-381 Prover Demo with Verification");
    println!("=======================================\n");
    
    // Use test configuration for quick demo
    let config = Config::test();
    println!("Using test configuration: n = 2^{}", config.log_n);
    
    // Setup phase
    println!("\n1. SETUP PHASE");
    println!("--------------");
    let setup = Setup::new(config);
    println!("✓ SRS generated with {} elements", setup.config.two_n());
    
    // Prover phase
    println!("\n2. PROVER PHASE");
    println!("---------------");
    let prover = Prover::new(setup.clone());
    let (commitment, polynomial_evals) = prover.prove();
    println!("✓ Commitment generated");
    
    // Create multiple opening proofs
    println!("\n3. OPENING PROOFS");
    println!("-----------------");
    let mut rng = test_rng();
    
    for i in 1..=3 {
        let eval_point = ark_bls12_381::Fr::rand(&mut rng);
        let opening_proof = prover.create_opening_proof(&polynomial_evals, eval_point);
        
        println!("\nOpening #{}", i);
        println!("  Point: {:?}", opening_proof.point);
        println!("  Evaluation: {:?}", opening_proof.evaluation);
        
        // Verify the opening
        let verifier = Verifier::new(setup.clone());
        let is_valid = verifier.verify_opening(&commitment, &opening_proof);
        println!("  Verification: {}", if is_valid { "✓ PASSED" } else { "✗ FAILED" });
    }
    
    // Test invalid proof
    println!("\n4. SECURITY TEST");
    println!("----------------");
    println!("Testing detection of invalid proofs...");
    
    let eval_point = ark_bls12_381::Fr::rand(&mut rng);
    let mut tampered_proof = prover.create_opening_proof(&polynomial_evals, eval_point);
    
    // Tamper with the evaluation
    tampered_proof.evaluation = ark_bls12_381::Fr::rand(&mut rng);
    
    let verifier = Verifier::new(setup);
    let is_valid = verifier.verify_opening(&commitment, &tampered_proof);
    println!("Tampered proof verification: {}", if is_valid { "✗ FAILED (Security breach!)" } else { "✓ PASSED (Attack detected)" });
    
    println!("\nDemo completed successfully!");
} 