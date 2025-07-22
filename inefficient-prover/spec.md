# BLS12-381 Protocol Specification

## Introduction

Use the Curve: BLS12-381 and assume n = 2¹⁷. We refer to base field as 𝔽q, and scalar field as 𝔽r.

## Setup

### 1. Generate a SRS in Lagrange basis

#### a. Generate random τ ∈ 𝔽r, and compute

{1, τ, τ², ... τ²ⁿ⁻¹}

#### b. Generate a random group element G ∈ 𝔾₁ of BLS12-381 and compute the SRS in monomial basis

[G]SRS = {G, τ · G, τ² · G, ... τ²ⁿ⁻¹ · G} ≡ {G₀, G₁, ... G₂ₙ₋₁}

#### c. Convert it into a Lagrange basis

### 2. Generate random polynomial with cᵢ ∈ 𝔽r in Lagrange basis

c₂ₙᵉᵛᵃˡ = {c₀, c₁, ... , c₂ₙ₋₁}

### 3. Prover gets [G]ᴸᵃᵍSRS and c₂ₙᵉᵛᵃˡ at the end of setup.

## Prover

### 1. **Witness**: Let xᵢ ∈ 𝔽r (you can use random field elements) for i = 0, 1 ... n - 1

fᵢ = Hash(xᵢ) ; fᵢ ∈ 𝔽r

### 2. Now we want to commit to the witness as follows. Convert the vector of fᵢ into a vector of length 2n using an FFT

f₂ₙᵉᵛᵃˡ = FFT₂ₙ[f₀, f₁, ... fₙ₋₁||0ₙ]

### 3. Compute the commitment

Gcomm = (c₂ₙᵉᵛᵃˡ ∘ f₂ₙᵉᵛᵃˡ)ᵀ · [G]ᴸᵃᵍSRS

The ∘ refers to Hadamard product, and the · is a Multiscalar multiplication.

---

**Implementation Note**: Below is an inefficient Prover protocol. Implement a Rust-code with the most efficient prover possible. You can use lambdaworks or arkworks.