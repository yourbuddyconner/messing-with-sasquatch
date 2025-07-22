# Exercises

A collection of cryptographic and mathematical exercises exploring zero-knowledge proofs and modular arithmetic.

## Projects

### [`inefficient-prover`](./inefficient-prover/)
A BLS12-381 elliptic curve prover implementation with polynomial commitments and opening proofs. 

Features:
- Polynomial commitment scheme using elliptic curves
- Opening proof generation and verification  
- Batch polynomial operations
- Integration tests for production-size problems

### [`modulo-machine`](./modulo-machine/) 
A digital circuit simulation that computes modular arithmetic operations. 

Features:
- 300-bit input, 256-bit prime modulus
- Clock and reset signal simulation
- GMP library backend for performance
- Batch processing support

## Building

This is a Cargo workspace. You can build all projects with:

```bash
cargo build --workspace
```

Or build individual projects:

```bash
cargo build -p bls12_381_prover
cargo build -p modulo-machine
```

## Testing

Run all tests:

```bash
cargo test --workspace
```

Run tests for specific projects:

```bash
cargo test -p bls12_381_prover
cargo test -p modulo-machine
```

## Running Examples

### Inefficient Prover
```bash
cargo run -p bls12_381_prover
cargo run --bin demo -p bls12_381_prover
```

### Modulo Machine
```bash
cargo run -p modulo-machine
```

## Dependencies

- **Arkworks**: Cryptographic primitives and elliptic curves
- **GMP/rug**: High-performance arbitrary precision arithmetic
- **Rayon**: Data parallelism for performance-critical operations