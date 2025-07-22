# Efficient BLS12-381 Polynomial Commitment Scheme

## Overview

This implements an optimized BLS12-381 polynomial commitment scheme following the specification in `spec.md`. The implementation includes both proving and verification capabilities with significant performance optimizations.

## Performance

Benchmark results across different scales:

| Size | Elements | Setup Time | Prover Time | Throughput | Verification |
|------|----------|------------|-------------|------------|--------------|
| n=2^10 | 1024 | 2.5s | 46ms | 21789 elem/s | ~1ms |
| n=2^12 | 4096 | 11.2s | 157ms | 25930 elem/s | ~1ms |
| n=2^14 | 16384 | 51.2s | 550ms | 29776 elem/s | ~2ms |
| n=2^16 | 65536 | 234.5s | 2043ms | 32070 elem/s | ~1ms |

The implementation scales to the specified production size of n = 2^17 with estimated ~4-5 second proving time.

## Key Features

- **Optimized witness generation**: Efficient binary serialization for hashing instead of string conversion
- **Parallel computation**: Multi-threaded setup and proving phases using rayon
- **Memory efficient**: Direct vector operations with minimal allocations
- **Coordinate optimization**: Strategic projective/affine coordinate management
- **State-of-the-art cryptography**: Uses arkworks library with Pippenger's MSM algorithm

## Protocol Implementation

The prover follows the specification exactly:
1. Generates witness f_i = Hash(x_i) for random x_i ∈ Fr
2. Converts to evaluation form using FFT with zero-padding to length 2n
3. Computes commitment as G_comm = (c_2n^eval ∘ f_2n^eval)^T · [G]^Lag_SRS
4. Supports KZG-style opening proofs with pairing-based verification

## Usage

```bash
# Run tests
cargo test

# Run demo (n = 2^10)
cargo run --bin demo --release

# Run production size (n = 2^17)
cargo run --release

# Run performance benchmarks
cargo run --bin benchmark --release
```

## Benchmarking

The `benchmark` binary tests performance across multiple scales (n = 2^10, 2^12, 2^14, 2^16) and produces a summary table:

```bash
cargo run --bin benchmark --release
```

Sample output:
```
Benchmark Results:
| Size | Elements | Setup Time | Prover Time | Throughput | Verification |
|------|----------|------------|-------------|------------|--------------|
| n=2^10 | 1024 | 2.5s | 46ms | 21789 elem/s | ~1ms |
| n=2^12 | 4096 | 11.2s | 157ms | 25930 elem/s | ~1ms |
| n=2^14 | 16384 | 51.2s | 550ms | 29776 elem/s | ~2ms |
| n=2^16 | 65536 | 234.5s | 2043ms | 32070 elem/s | ~1ms |
```

## Dependencies

- `ark-bls12-381`: BLS12-381 curve operations
- `ark-ec`: Elliptic curve and pairing operations  
- `ark-ff`: Finite field arithmetic
- `ark-poly`: Polynomial operations and FFT
- `ark-serialize`: Efficient serialization
- `rayon`: Parallel processing
- `sha2`: SHA-256 hashing 