use bls12_381_prover::*;

fn main() {
    println!("BLS12-381 Efficient Prover Implementation");
    println!("=========================================");
    
    let config = Config::production();
    println!("Parameters: n = 2^{}, curve = BLS12-381", config.log_n);
    
    // Setup phase
    let setup = Setup::new(config);
    
    // Prover phase
    let prover = Prover::new(setup.clone());
    let (commitment, polynomial_evals) = prover.prove();
    
    println!("\nFinal commitment: {:?}", commitment);
    
    // Create opening proof for a random point
    let mut rng = test_rng();
    let eval_point = Fr::rand(&mut rng);
    let opening_proof = prover.create_opening_proof(&polynomial_evals, eval_point);
    
    println!("\nOpening proof created for point: {:?}", eval_point);
    println!("Claimed evaluation: {:?}", opening_proof.evaluation);
    
    // Verification phase
    let verifier = Verifier::new(setup);
    let is_valid = verifier.verify_opening(&commitment, &opening_proof);
    
    println!("\nProtocol execution completed successfully!");
    println!("Opening proof verification: {}", if is_valid { "PASSED ✓" } else { "FAILED ✗" });
}
