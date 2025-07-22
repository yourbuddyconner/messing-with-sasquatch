# Modulo Machine

A Rust implementation of a digital circuit that computes `X mod P` using the GMP library backend for improved performance over pure Rust big integer implementations.

## Specifications

- **X**: 300-bit input value
- **P**: 256-bit prime number = `104899928942039473597645237135751317405745389583683433800060134911610808289117`

## Interface

The machine simulates a synchronous digital circuit with:
- **Clock** (1-bit): Rising edge triggers computation
- **Reset** (1-bit): Clears the output register
- **X** (300-bit): Input value to compute modulo of
- **O** (256-bit): Output result of `X mod P`

## Implementation Details

- Uses GMP library backend (GNU Multiple Precision Arithmetic Library)
- Returns references instead of clones to reduce allocations
- Supports batch processing for multiple operations
- GMP automatically uses Montgomery reduction when beneficial for the modulus size

## Usage

### Running the Demo

```bash
cd modulo-machine
cargo run
```

### Running Tests

```bash
cargo test
```

### Using the Library

```rust
use modulo_machine::ModuloMachine;
use rug::Integer;

let mut machine = ModuloMachine::new();

// Create input and compute X mod P
let x = ModuloMachine::create_input_u64(12345);
let result = machine.tick(true, false, &x);  // clock=true, reset=false
println!("Result: {}", result);

// Batch processing multiple inputs
let inputs = vec![
    (true, false, &ModuloMachine::create_input_u64(1111)),
    (true, false, &ModuloMachine::create_input_u64(2222)),
    (true, false, &ModuloMachine::create_input_u64(3333)),
];
let results = machine.process_batch(&inputs);

// Create large numbers efficiently
let big_input = ModuloMachine::create_large_input(299, 123456789); // 2^299 + offset
```

## Features

- GMP backend for modular arithmetic operations
- Reference-based API to reduce allocations
- Batch processing support
- Helper methods for creating large integers
- Input validation using bit counting

## Architecture

The `ModuloMachine` struct maintains:
- The prime modulus P (loaded from constant)
- Current output register
- Previous clock state for edge detection

The computation is triggered only on the rising edge of the clock signal, simulating real hardware behavior.

## Dependencies

- `rug`: GMP library bindings for Rust 