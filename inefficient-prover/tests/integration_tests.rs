use bls12_381_prover::*;
use ark_ff::UniformRand;
use ark_std::test_rng;

#[test]
fn test_setup() {
    let config = Config::test();
    let setup = Setup::new(config);
    assert_eq!(setup.srs_lagrange_g1.len(), setup.config.two_n());
    assert_eq!(setup.srs_monomial_g1.len(), setup.config.two_n());
    assert_eq!(setup.c_eval.len(), setup.config.two_n());
}

#[test]
fn test_prover() {
    let config = Config::test();
    let setup = Setup::new(config);
    let prover = Prover::new(setup);
    let (commitment, _) = prover.prove();
    // Verify commitment is not the point at infinity
    assert!(!commitment.is_zero());
}

#[test]
fn test_opening_proof() {
    let config = Config::test();
    let setup = Setup::new(config.clone());
    let prover = Prover::new(setup.clone());
    let (commitment, polynomial_evals) = prover.prove();
    
    // Create and verify opening proof
    let mut rng = test_rng();
    let eval_point = Fr::rand(&mut rng);
    let opening_proof = prover.create_opening_proof(&polynomial_evals, eval_point);
    
    let verifier = Verifier::new(setup);
    assert!(verifier.verify_opening(&commitment, &opening_proof));
}

#[test]
fn test_invalid_opening_proof() {
    let config = Config::test();
    let setup = Setup::new(config.clone());
    let prover = Prover::new(setup.clone());
    let (commitment, polynomial_evals) = prover.prove();
    
    // Create valid opening proof
    let mut rng = test_rng();
    let eval_point = Fr::rand(&mut rng);
    let mut opening_proof = prover.create_opening_proof(&polynomial_evals, eval_point);
    
    // Tamper with the evaluation
    opening_proof.evaluation = Fr::rand(&mut rng);
    
    let verifier = Verifier::new(setup);
    assert!(!verifier.verify_opening(&commitment, &opening_proof));
}

#[test]
fn test_production_size() {
    // Just verify the configuration is correct
    let config = Config::production();
    assert_eq!(config.n(), 1 << 17);
    assert_eq!(config.two_n(), 2 << 17);
} 