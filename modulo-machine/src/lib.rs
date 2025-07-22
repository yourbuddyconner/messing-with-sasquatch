use rug::{Integer, Assign};

/// The 256-bit prime P from the specification
pub const P_STR: &str = "104899928942039473597645237135751317405745389583683433800060134911610808289117";

/// Modulo Machine using GMP library for modular arithmetic
pub struct ModuloMachine {
    /// The prime modulus P (256-bit)
    p: Integer,
    /// Current output (256-bit) 
    output: Integer,
    /// Internal state for clock simulation
    clk_prev: bool,
    /// Pre-computed values for fast modular arithmetic
    /// Cached for repeated operations with same modulus
    _p_bits: u32,
}

impl ModuloMachine {
    /// Create a new modulo machine instance
    pub fn new() -> Self {
        let p = Integer::from_str_radix(P_STR, 10).expect("Failed to parse prime P");
        let p_bits = p.significant_bits();
        
        Self {
            output: Integer::new(),
            p,
            clk_prev: false,
            _p_bits: p_bits,
        }
    }

    /// Reset the machine (clear output)
    pub fn reset(&mut self) {
        self.output.assign(0);
        self.clk_prev = false;
    }

    /// Process one clock cycle
    /// - clk: clock input (1 bit)
    /// - reset: reset input (1 bit) 
    /// - x: input value (300 bits max)
    /// Returns: current output (256 bits max)
    pub fn tick(&mut self, clk: bool, reset: bool, x: &Integer) -> &Integer {
        // Handle reset
        if reset {
            self.reset();
            return &self.output;
        }

        // Process on rising edge of clock
        if clk && !self.clk_prev {
            // Compute X mod P using GMP's modular arithmetic
            self.output.assign(x % &self.p);
        }

        self.clk_prev = clk;
        &self.output
    }

    /// Batch processing for multiple inputs
    /// Processes multiple clock cycles in one call
    pub fn process_batch(&mut self, inputs: &[(bool, bool, &Integer)]) -> Vec<Integer> {
        let mut results = Vec::with_capacity(inputs.len());
        
        for &(clk, reset, x) in inputs {
            let result = self.tick(clk, reset, x);
            results.push(result.clone());
        }
        
        results
    }

    /// Get current output without processing a clock tick
    pub fn get_output(&self) -> &Integer {
        &self.output
    }

    /// Get the prime modulus P
    pub fn get_prime(&self) -> &Integer {
        &self.p
    }

    /// Validate that input X is within 300-bit limit
    pub fn validate_input_size(x: &Integer) -> bool {
        // 300 bits can represent numbers up to 2^300 - 1
        x.significant_bits() <= 300
    }

    /// Validate that output is within 256-bit limit
    pub fn validate_output_size(output: &Integer) -> bool {
        // 256 bits can represent numbers up to 2^256 - 1  
        output.significant_bits() <= 256
    }

    /// Create input from string
    pub fn create_input(s: &str, radix: i32) -> Result<Integer, rug::integer::ParseIntegerError> {
        Integer::from_str_radix(s, radix)
    }

    /// Create input from u64
    pub fn create_input_u64(val: u64) -> Integer {
        Integer::from(val)
    }

    /// Create large test inputs
    pub fn create_large_input(base_power: u32, offset: u64) -> Integer {
        // Creates 2^base_power + offset
        (Integer::from(1) << base_power) + offset
    }
}

impl Default for ModuloMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_creation() {
        let machine = ModuloMachine::new();
        assert_eq!(*machine.get_output(), 0);
        
        // Verify P is loaded correctly
        let expected_p = Integer::from_str_radix(P_STR, 10).unwrap();
        assert_eq!(machine.get_prime(), &expected_p);
    }

    #[test]
    fn test_reset_functionality() {
        let mut machine = ModuloMachine::new();
        
        // Set some output first
        let x = Integer::from(12345u32);
        machine.tick(true, false, &x);
        assert_ne!(*machine.get_output(), 0);
        
        // Reset should clear output
        let zero = Integer::from(0);
        machine.tick(false, true, &zero);
        assert_eq!(*machine.get_output(), 0);
    }

    #[test]
    fn test_basic_modulo_operation() {
        let mut machine = ModuloMachine::new();
        let p = machine.get_prime().clone();
        
        // Test with a number smaller than P
        let x_small = Integer::from(12345u32);
        let result = machine.tick(true, false, &x_small);
        assert_eq!(*result, x_small); // Should be unchanged since x < P
        
        // Test with P itself - need to cycle clock first
        let zero = Integer::from(0);
        machine.tick(false, false, &zero); // Clock low
        let result = machine.tick(true, false, &p); // Clock high (rising edge)
        assert_eq!(*result, 0); // P mod P = 0
        
        // Test with P + 1 - need to cycle clock first
        machine.tick(false, false, &zero); // Clock low
        let x_large = Integer::from(&p + 1);
        let result = machine.tick(true, false, &x_large); // Clock high (rising edge)
        assert_eq!(*result, 1); // (P + 1) mod P = 1
    }

    #[test]
    fn test_input_size_validation() {
        // Test valid 300-bit input
        let max_300_bit = ModuloMachine::create_large_input(300, 0) - 1;
        assert!(ModuloMachine::validate_input_size(&max_300_bit));
        
        // Test invalid 301-bit input  
        let min_301_bit = ModuloMachine::create_large_input(300, 0);
        assert!(!ModuloMachine::validate_input_size(&min_301_bit));
    }

    #[test]
    fn test_output_size_validation() {
        let machine = ModuloMachine::new();
        let p = machine.get_prime();
        
        // Output should always be < P, so within 256 bits
        assert!(ModuloMachine::validate_output_size(p));
        let p_minus_one = Integer::from(p - 1);
        assert!(ModuloMachine::validate_output_size(&p_minus_one));
    }

    #[test]
    fn test_batch_processing() {
        let mut machine = ModuloMachine::new();
        
        // Create test inputs
        let input1 = Integer::from(12345u64);
        let input2 = Integer::from(67890u64);
        let input3 = Integer::from(0u64);
        let input4 = Integer::from(99999u64);
        
        // Need to properly cycle clock for each operation
        let inputs = vec![
            (true, false, &input1),  // Rising edge - should process
            (false, false, &input1), // Clock low
            (true, false, &input2),  // Rising edge - should process input2
            (false, true, &input3),  // Reset while clock low
            (true, false, &input4),  // Rising edge after reset - should process input4
        ];
        
        let results = machine.process_batch(&inputs);
        assert_eq!(results.len(), 5);
        assert_eq!(results[0], 12345); // First input processed
        assert_eq!(results[1], 12345); // Clock low, output unchanged
        assert_eq!(results[2], 67890); // Second input processed
        assert_eq!(results[3], 0);     // After reset
        assert_eq!(results[4], 99999); // Fourth input processed
    }

    #[test]
    fn test_performance_helpers() {
        // Test optimized input creation methods
        let from_u64 = ModuloMachine::create_input_u64(12345);
        assert_eq!(from_u64, 12345);
        
        let from_string = ModuloMachine::create_input("12345", 10).unwrap();
        assert_eq!(from_string, 12345);
        
        let large_input = ModuloMachine::create_large_input(10, 123);
        assert_eq!(large_input, 1024 + 123); // 2^10 + 123
    }
} 