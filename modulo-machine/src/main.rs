use modulo_machine::ModuloMachine;
use rug::Integer;

fn main() {
    println!("Modulo Machine Demo");
    println!("===================");
    
    // Create a new modulo machine
    let mut machine = ModuloMachine::new();
    
    println!("Prime P: {}", machine.get_prime());
    println!("P has {} bits", machine.get_prime().significant_bits());
    println!();
    
    // Test 1: Simple small number
    println!("Test 1: X = 12345");
    let x1 = ModuloMachine::create_input_u64(12345);
    let result1 = machine.tick(true, false, &x1);
    println!("Input:  {} ({} bits)", x1, x1.significant_bits());
    println!("Output: {} ({} bits)", result1, result1.significant_bits());
    println!();
    
    // Test 2: Number equal to P
    println!("Test 2: X = P");
    let x2 = machine.get_prime().clone();
    let zero = Integer::from(0);
    machine.tick(false, false, &zero); // Clock low
    let result2 = machine.tick(true, false, &x2); // Clock high (rising edge)
    println!("Input:  {} ({} bits)", x2, x2.significant_bits());
    println!("Output: {} ({} bits)", result2, result2.significant_bits());
    println!();
    
    // Test 3: Number larger than P
    println!("Test 3: X = P + 100000");
    let x3 = Integer::from(machine.get_prime() + 100000u32);
    machine.tick(false, false, &zero); // Clock low
    let result3 = machine.tick(true, false, &x3); // Clock high (rising edge)
    println!("Input:  {} ({} bits)", x3, x3.significant_bits());
    println!("Output: {} ({} bits)", result3, result3.significant_bits());
    println!();
    
    // Test 4: Large 300-bit number
    println!("Test 4: Large 300-bit number");
    let x4 = ModuloMachine::create_large_input(299, 123456789);
    machine.tick(false, false, &zero); // Clock low
    let result4 = machine.tick(true, false, &x4); // Clock high (rising edge)
    println!("Input:  {} ({} bits)", x4, x4.significant_bits());
    println!("Output: {} ({} bits)", result4, result4.significant_bits());
    println!();
    
    // Test 5: Reset functionality
    println!("Test 5: Reset functionality");
    println!("Before reset - Output: {}", machine.get_output());
    machine.tick(false, true, &zero); // Reset
    println!("After reset  - Output: {}", machine.get_output());
    println!();
    
    // Test 6: Clock edge behavior
    println!("Test 6: Clock edge behavior");
    let x6 = ModuloMachine::create_input_u64(555555);
    
    // Clock low -> high (should process)
    println!("Clock low -> high:");
    machine.tick(false, false, &x6); // Clock low
    println!("  Output after low:  {}", machine.get_output());
    let result6a = machine.tick(true, false, &x6); // Clock high (rising edge)
    println!("  Output after high: {}", result6a);
    
    // Clock high -> high (should not process again)
    println!("Clock high -> high:");
    let different_x = ModuloMachine::create_input_u64(999999);
    let result6b = machine.tick(true, false, &different_x); // Clock still high
    println!("  Output (unchanged): {}", result6b);
    println!();
    
    // Test 7: Batch Processing
    println!("Test 7: Batch Processing");
    let batch1 = ModuloMachine::create_input_u64(1111);
    let batch2 = ModuloMachine::create_input_u64(2222);
    let batch3 = ModuloMachine::create_input_u64(3333);
    let batch_reset = Integer::from(0);
    let batch4 = ModuloMachine::create_input_u64(4444);
    
    let batch_inputs = vec![
        (true, false, &batch1),
        (true, false, &batch2), 
        (true, false, &batch3),
        (false, true, &batch_reset), // Reset
        (true, false, &batch4),
    ];
    
    let batch_results = machine.process_batch(&batch_inputs);
    println!("Batch processed {} inputs:", batch_results.len());
    for (i, result) in batch_results.iter().enumerate() {
        println!("  Result {}: {}", i + 1, result);
    }
    println!();
    
    // Validation tests
    println!("📋 Validation Tests");
    println!("==================");
    
    // Test input size validation
    let max_300_bit = ModuloMachine::create_large_input(300, 0) - 1;
    let too_large = ModuloMachine::create_large_input(300, 0);
    
    println!("300-bit max valid:   {}", ModuloMachine::validate_input_size(&max_300_bit));
    println!("301-bit (too large): {}", ModuloMachine::validate_input_size(&too_large));
    
    // Test output size validation
    let p = machine.get_prime().clone();
    println!("Output size for P:     {}", ModuloMachine::validate_output_size(&p));
    let p_minus_one = Integer::from(&p - 1);
    println!("Output size for P-1:   {}", ModuloMachine::validate_output_size(&p_minus_one));
    
    println!("\n✅ Demo completed!");
} 