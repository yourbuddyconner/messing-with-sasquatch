use ark_bls12_381::{Fr, G1Affine, G1Projective, G2Affine, G2Projective, Bls12_381};
use ark_ec::{CurveGroup, VariableBaseMSM, AffineRepr, pairing::Pairing};
use ark_ff::{UniformRand, Zero, One, PrimeField};
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain, univariate::DensePolynomial, Polynomial, DenseUVPolynomial};
use ark_std::test_rng;
use ark_serialize::CanonicalSerialize;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::time::Instant;

/// n = 2^17 as specified for production
pub const PRODUCTION_LOG_N: usize = 17;

/// Configuration for the protocol
#[derive(Clone)]
pub struct Config {
    pub log_n: usize,
}

impl Config {
    pub fn production() -> Self {
        Config { log_n: PRODUCTION_LOG_N }
    }
    
    pub fn test() -> Self {
        // Use a much smaller size for tests (2^10 = 1024)
        Config { log_n: 10 }
    }
    
    pub fn n(&self) -> usize {
        1 << self.log_n
    }
    
    pub fn two_n(&self) -> usize {
        2 * self.n()
    }
}

/// Setup phase - generates SRS in Lagrange basis
#[derive(Clone)]
pub struct Setup {
    /// SRS in Lagrange basis for G1 (keep in projective for efficiency)
    pub srs_lagrange_g1: Vec<G1Projective>,
    /// SRS in monomial basis for G1 (needed for opening proofs)
    pub srs_monomial_g1: Vec<G1Affine>,
    /// G2 generator and tau*G2 for pairing checks
    pub g2: G2Affine,
    pub tau_g2: G2Affine,
    /// Random polynomial evaluations c_i
    pub c_eval: Vec<Fr>,
    /// Configuration
    pub config: Config,
}

impl Setup {
    pub fn new(config: Config) -> Self {
        println!("Starting setup phase for n = 2^{}...", config.log_n);
        let start = Instant::now();
        
        let mut rng = test_rng();
        let two_n = config.two_n();
        
        // 1. Generate random τ ∈ Fr
        let tau = Fr::rand(&mut rng);
        
        // 2. Generate powers of τ efficiently using parallel windowing
        println!("Computing powers of τ...");
        let tau_powers = Self::compute_powers_parallel(tau, two_n);
        
        // 3. Generate random G ∈ G1 and H ∈ G2
        let g1 = G1Projective::rand(&mut rng);
        let g2 = G2Projective::rand(&mut rng);
        
        // 4. Compute SRS in monomial basis using parallel scalar multiplication
        println!("Computing SRS in monomial basis...");
        let srs_monomial: Vec<G1Projective> = tau_powers
            .par_iter()
            .map(|tau_i| g1 * tau_i)
            .collect();
        
        // Convert monomial basis to affine only for what we need for opening proofs
        let srs_monomial_g1: Vec<G1Affine> = srs_monomial
            .par_iter()
            .map(|p| p.into_affine())
            .collect();
        
        // 5. Convert to Lagrange basis using FFT (keep in projective)
        println!("Converting to Lagrange basis...");
        let domain = Radix2EvaluationDomain::<Fr>::new(two_n).unwrap();
        let srs_lagrange = Self::monomial_to_lagrange(&srs_monomial, &domain);
        
        // 6. Generate random polynomial evaluations in parallel
        let c_eval: Vec<Fr> = (0..two_n)
            .into_par_iter()
            .map(|_| {
                let mut local_rng = test_rng();
                Fr::rand(&mut local_rng)
            })
            .collect();
        
        // 7. Compute G2 elements for verification
        let tau_g2 = (g2 * tau).into_affine();
        
        println!("Setup completed in {:?}", start.elapsed());
        
        Setup {
            srs_lagrange_g1: srs_lagrange,
            srs_monomial_g1,
            g2: g2.into_affine(),
            tau_g2,
            c_eval,
            config,
        }
    }
    
    /// Compute powers of τ efficiently using parallel computation
    fn compute_powers_parallel(tau: Fr, count: usize) -> Vec<Fr> {
        if count <= 1 {
            return vec![Fr::one()];
        }
        
        // For small counts, use sequential method
        if count <= 1024 {
            let mut powers = Vec::with_capacity(count);
            powers.push(Fr::one());
            for i in 1..count {
                powers.push(powers[i - 1] * tau);
            }
            return powers;
        }
        
        // For larger counts, use windowing approach
        let window_size = 256;
        let num_windows = (count + window_size - 1) / window_size;
        
        // Compute tau^window_size, tau^(2*window_size), etc.
        let tau_window = {
            let mut result = Fr::one();
            for _ in 0..window_size {
                result *= tau;
            }
            result
        };
        
        let mut window_bases = vec![Fr::one()];
        for i in 1..num_windows {
            window_bases.push(window_bases[i - 1] * tau_window);
        }
        
        // Compute all powers in parallel
        let powers: Vec<Fr> = (0..count)
            .into_par_iter()
            .map(|i| {
                let window_idx = i / window_size;
                let offset = i % window_size;
                let base = window_bases[window_idx];
                if offset == 0 {
                    base
                } else {
                    let mut tau_offset = Fr::one();
                    for _ in 0..offset {
                        tau_offset *= tau;
                    }
                    base * tau_offset
                }
            })
            .collect();
        
        powers
    }
    
    /// Convert SRS from monomial to Lagrange basis using FFT
    fn monomial_to_lagrange(
        srs_monomial: &[G1Projective],
        domain: &Radix2EvaluationDomain<Fr>,
    ) -> Vec<G1Projective> {
        // We need to perform an IFFT on the group elements
        let mut srs_lagrange = srs_monomial.to_vec();
        
        // The conversion is essentially computing L_i(τ) * G for each Lagrange basis polynomial L_i
        // This can be done efficiently using the FFT structure
        domain.ifft_in_place(&mut srs_lagrange);
        
        srs_lagrange
    }
}

/// Opening proof for polynomial evaluation
#[derive(Clone, Debug)]
pub struct OpeningProof {
    /// The evaluation point
    pub point: Fr,
    /// The claimed evaluation
    pub evaluation: Fr,
    /// The proof element (quotient polynomial commitment)
    pub proof: G1Affine,
}

/// Prover - generates witness and commitment
pub struct Prover {
    setup: Setup,
}

impl Prover {
    pub fn new(setup: Setup) -> Self {
        Prover { setup }
    }
    
    pub fn prove(&self) -> (G1Affine, Vec<Fr>) {
        println!("Starting prover phase...");
        let start = Instant::now();
        
        let mut rng = test_rng();
        let n = self.setup.config.n();
        let two_n = self.setup.config.two_n();
        
        // 1. Generate witness: random x_i ∈ Fr for i = 0, 1, ..., n-1
        let x_values: Vec<Fr> = (0..n).map(|_| Fr::rand(&mut rng)).collect();
        
        // 2. Compute f_i = Hash(x_i)
        let f_values: Vec<Fr> = x_values
            .par_iter()
            .map(|x| {
                let mut hasher = Sha256::new();
                // Use canonical serialization instead of string conversion
                let mut bytes = Vec::new();
                x.serialize_compressed(&mut bytes).unwrap();
                hasher.update(&bytes);
                let hash = hasher.finalize();
                // Convert hash to field element
                Fr::from_be_bytes_mod_order(&hash)
            })
            .collect();
        
        // 3. Convert to length 2n using FFT (pad with zeros)
        println!("Computing FFT...");
        let mut f_2n_eval = f_values;
        f_2n_eval.resize(two_n, Fr::zero());
        
        let domain = Radix2EvaluationDomain::<Fr>::new(two_n).unwrap();
        domain.fft_in_place(&mut f_2n_eval);
        
        // 4. Compute commitment: G_comm = (c_2n^eval ∘ f_2n^eval)^T · [G]^Lag_SRS
        println!("Computing commitment...");
        
        // Hadamard product - keep parallelized
        let hadamard_product: Vec<Fr> = self.setup.c_eval
            .par_iter()
            .zip(f_2n_eval.par_iter())
            .map(|(c, f)| *c * f)
            .collect();
        
        // Multi-scalar multiplication (MSM) - convert to affine only when needed
        let srs_lagrange_affine: Vec<G1Affine> = self.setup.srs_lagrange_g1
            .par_iter()
            .map(|p| p.into_affine())
            .collect();
        
        let commitment = Self::efficient_msm(&srs_lagrange_affine, &hadamard_product);
        
        println!("Prover completed in {:?}", start.elapsed());
        
        (commitment.into_affine(), hadamard_product)
    }
    
    /// Create an opening proof for a specific evaluation point
    pub fn create_opening_proof(
        &self,
        polynomial_evals: &[Fr],
        point: Fr,
    ) -> OpeningProof {
        println!("Creating opening proof for point {:?}", point);
        
        // Convert evaluations back to coefficient form
        let domain = Radix2EvaluationDomain::<Fr>::new(polynomial_evals.len()).unwrap();
        let mut coeffs = polynomial_evals.to_vec();
        domain.ifft_in_place(&mut coeffs);
        
        // Create polynomial from coefficients
        let poly = DensePolynomial::from_coefficients_vec(coeffs);
        
        // Evaluate polynomial at the point
        let evaluation = poly.evaluate(&point);
        
        // Compute quotient polynomial: q(x) = (p(x) - p(z)) / (x - z)
        let numerator = &poly - &DensePolynomial::from_coefficients_vec(vec![evaluation]);
        let denominator = DensePolynomial::from_coefficients_vec(vec![-point, Fr::one()]);
        let quotient = &numerator / &denominator;
        
        // Commit to quotient polynomial
        let quotient_coeffs = quotient.coeffs();
        let proof = if quotient_coeffs.len() <= self.setup.srs_monomial_g1.len() {
            Self::efficient_msm(
                &self.setup.srs_monomial_g1[..quotient_coeffs.len()],
                quotient_coeffs,
            )
            .into_affine()
        } else {
            panic!("Quotient polynomial degree too high");
        };
        
        OpeningProof {
            point,
            evaluation,
            proof,
        }
    }
    
    /// Efficient multi-scalar multiplication using arkworks' optimized implementation
    fn efficient_msm(bases: &[G1Affine], scalars: &[Fr]) -> G1Projective {
        // arkworks provides highly optimized MSM using Pippenger's algorithm
        // with parallelization and other optimizations
        G1Projective::msm(bases, scalars).unwrap()
    }
}

/// Verifier - verifies commitments and opening proofs
pub struct Verifier {
    setup: Setup,
}

impl Verifier {
    pub fn new(setup: Setup) -> Self {
        Verifier { setup }
    }
    
    /// Verify an opening proof using pairing check
    pub fn verify_opening(
        &self,
        commitment: &G1Affine,
        proof: &OpeningProof,
    ) -> bool {
        println!("Verifying opening proof...");
        
        // Pairing check: e(C - v*G, H) = e(π, τ*H - z*H)
        // Where:
        // - C is the commitment
        // - v is the claimed evaluation
        // - G is the generator (first SRS element)
        // - π is the proof
        // - z is the evaluation point
        
        let g1_gen = self.setup.srs_monomial_g1[0];
        
        // Left side: C - v*G
        let left = commitment.into_group() - g1_gen * proof.evaluation;
        
        // Right side G2: τ*H - z*H
        let right_g2 = self.setup.tau_g2.into_group() - self.setup.g2 * proof.point;
        
        // Perform pairing check
        let pairing1 = Bls12_381::pairing(left, self.setup.g2);
        let pairing2 = Bls12_381::pairing(proof.proof, right_g2);
        
        let result = pairing1 == pairing2;
        println!("Verification result: {}", result);
        
        result
    }
}
