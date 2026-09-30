//! 6502 Implementation for the Klaus Dormann Tests.
//!
//! The Klaus Dormann tests are the canonical 6502 emulator tests.
//! This module implements a 6502 emulator based on the core module that
//! complies with its requirements to verify that the 6502 does exactly what
//! it should.
//!
//! The test source is included in this repo so that it can be run directly.
//!
//! ## Memory Mapped I/O
//! Address 0xf004 is used for input and 0xf001 is used for output.
//! The interrupt test uses address 0xbffc as a feedback register whose bit 0
//! drives IRQ and whose bit 1 drives NMI.

use akai6502::{Bus, CPU, DMATransferStatus, MAX_ADDRESSABLE_MEMORY};
use std::io::Read;
use std::path::Path;

struct KlausDormann6502 {
    mem: [u8; MAX_ADDRESSABLE_MEMORY],
}

const FUNCTIONAL_TEST_START: usize = 0x0400;
const FUNCTIONAL_TEST_SUCCESS: u16 = 0x3489;

const INTERRUPT_TEST_START: u16 = 0x0400;
const INTERRUPT_TEST_SUCCESS: u16 = 0x06f5;
const INTERRUPT_TEST_FEEDBACK: usize = 0xbffc;
const INTERRUPT_TEST_IRQ: u8 = 0b0000_0001;
const INTERRUPT_TEST_NMI: u8 = 0b0000_0010;

// These constants describe the memory interface exposed by
// `6502_decimal_test.a65`. They are addresses in the emulated memory, not
// hardcoded operands or expected results. The assembly program writes its
// current operands and results here while testing every operand pair with both
// carry inputs.

// The address at which the decimal test is loaded and begins execution.
const DECIMAL_TEST_START: usize = 0x0200;

// The operand pair currently being tested.
const DECIMAL_TEST_N1: usize = 0x0000;
const DECIMAL_TEST_N2: usize = 0x0001;

// The accumulator and complete status byte produced by the emulated CPU.
// Addresses $0002 and $0003 hold intermediate binary results that are not
// needed by the Rust failure message.
const DECIMAL_TEST_ACTUAL_A    : usize = 0x0004;
const DECIMAL_TEST_ACTUAL_FLAGS: usize = 0x0005;

// The accumulator and flags calculated by the test's reference algorithm.
// Each flag address contains a complete status-byte snapshot, not a boolean,
// so the relevant bit is masked from its normal position below.
const DECIMAL_TEST_PREDICTED_A: usize = 0x0006;
const DECIMAL_TEST_PREDICTED_N: usize = 0x0007;
const DECIMAL_TEST_PREDICTED_V: usize = 0x0008;
const DECIMAL_TEST_PREDICTED_Z: usize = 0x0009;
const DECIMAL_TEST_PREDICTED_C: usize = 0x000a;

// This remains one after a mismatch. It is cleared only if every case passes.
const DECIMAL_TEST_ERROR: usize = 0x000b;

// With reporting disabled, each functional-test assertion uses a conditional
// branch back to itself as its failure trap. The branch opcode therefore tells
// us which flag condition caused the test to stop.
fn klaus_trap_reason(opcode: u8) -> &'static str {
    match opcode {
        0xf0 => "Z flag set when it should be clear",
        0xd0 => "Z flag clear when it should be set",
        0xb0 => "C flag set when it should be clear",
        0x90 => "C flag clear when it should be set",
        0x30 => "N flag set when it should be clear",
        0x10 => "N flag clear when it should be set",
        0x70 => "V flag set when it should be clear",
        0x50 => "V flag clear when it should be set",
        0x4c => "generic test trap",
        _ => "unexpected stopping instruction",
    }
}

#[test]
fn test_6502_functional() {
    let mut bus = KlausDormann6502::new();
    bus.load_program_binary_at(
        "./tests/6502_functional_test_copy.bin",
        FUNCTIONAL_TEST_START,
    )
    .expect("Failed to load the functional test binary");
    let mut cpu = CPU::<KlausDormann6502>::new_dormann();
    cpu.run(&mut bus);

    let trap_address = cpu.pc();
    let trap_reason = klaus_trap_reason(bus.mem[trap_address as usize]);

    assert_eq!(
        trap_address,
        FUNCTIONAL_TEST_SUCCESS,
        concat!(
            "Klaus Dormann functional test failed\n",
            "CPU: {}\n",
            "Trap address: ${:04X}\n",
            "Mismatch: {}\n",
            "Listing key: {:04x} :",
        ),
        cpu,
        trap_address,
        trap_reason,
        trap_address,
    );
    println!("{cpu}")
}

#[test]
fn test_6502_interrupts() {
    let mut bus = KlausDormann6502::new();
    bus.load_program_intel_hex("./tests/6502_interrupt_test.hex")
        .expect("Failed to load the interrupt test image");

    let mut cpu = CPU::<KlausDormann6502>::new();
    cpu.set_pc(INTERRUPT_TEST_START);
    cpu.run(&mut bus);

    let trap_address = cpu.pc();
    let trap_reason = klaus_trap_reason(bus.mem[trap_address as usize]);

    assert_eq!(
        trap_address,
        INTERRUPT_TEST_SUCCESS,
        concat!(
            "Klaus Dormann interrupt test failed\n",
            "CPU: {}\n",
            "Trap address: ${:04X}\n",
            "Feedback IRQ/NMI: {:02b}\n",
            "Mismatch: {}\n",
            "Listing key: {:04x} :",
        ),
        cpu,
        trap_address,
        bus.mem[INTERRUPT_TEST_FEEDBACK]
            & (INTERRUPT_TEST_IRQ | INTERRUPT_TEST_NMI),
        trap_reason,
        trap_address,
    );
}

#[test]
fn test_6502_decimal() {
    let mut bus = KlausDormann6502::new();
    bus.load_program_binary_at(
        "./tests/6502_decimal_test.bin",
        DECIMAL_TEST_START,
    )
    .expect("Failed to load the decimal test binary");

    let mut cpu = CPU::<KlausDormann6502>::new();
    cpu.set_pc(DECIMAL_TEST_START as u16);
    cpu.run(&mut bus);

    let actual_a = bus.mem[DECIMAL_TEST_ACTUAL_A];
    let predicted_a = bus.mem[DECIMAL_TEST_PREDICTED_A];
    let predicted_flags = (bus.mem[DECIMAL_TEST_PREDICTED_N] & 0x80)
        | (bus.mem[DECIMAL_TEST_PREDICTED_V] & 0x40)
        | (bus.mem[DECIMAL_TEST_PREDICTED_Z] & 0x02)
        | (bus.mem[DECIMAL_TEST_PREDICTED_C] & 0x01);
    let actual_nvzc = ((bus.mem[DECIMAL_TEST_ACTUAL_FLAGS] & 0xc0) >> 4)
        | (bus.mem[DECIMAL_TEST_ACTUAL_FLAGS] & 0x03);
    let predicted_nvzc = ((predicted_flags & 0xc0) >> 4)
        | (predicted_flags & 0x03);
    let differing_flags = actual_nvzc ^ predicted_nvzc;
    let flag_markers: String = [0x08, 0x04, 0x02, 0x01]
        .iter()
        .map(|flag| if differing_flags & flag != 0 { '^' } else { ' ' })
        .collect();

    let mut mismatches = Vec::new();
    if actual_a != predicted_a {
        mismatches.push("accumulator");
    }
    let flags = [
        ("N flag", 0x08),
        ("V flag", 0x04),
        ("Z flag", 0x02),
        ("C flag", 0x01),
    ];
    for (name, flag) in flags {
        if differing_flags & flag != 0 {
            mismatches.push(name);
        }
    }
    let mismatches = if mismatches.is_empty() {
        "unknown".to_owned()
    } else {
        mismatches.join(", ")
    };

    assert_eq!(
        bus.mem[DECIMAL_TEST_ERROR],
        0,
        concat!(
            "Decimal mode test failed\n",
            "CPU: {}\n",
            "Operands: N1=${:02X}, N2=${:02X}\n",
            "Accumulator: actual=${:02X}, predicted=${:02X}\n",
            "\n",
            "             NVZC\n",
            "Actual:      {:04b}\n",
            "Predicted:   {:04b}\n",
            "             {}\n",
            "Mismatch: {}",
        ),
        cpu,
        bus.mem[DECIMAL_TEST_N1],
        bus.mem[DECIMAL_TEST_N2],
        actual_a,
        predicted_a,
        actual_nvzc,
        predicted_nvzc,
        flag_markers,
        mismatches,
    );
}

impl KlausDormann6502 {
    fn new() -> Self {
        KlausDormann6502 {
            mem: [0; MAX_ADDRESSABLE_MEMORY],
        }
    }

    /// Loads a raw binary file into memory starting at `start`.
    fn load_program_binary_at(
        &mut self,
        path: impl AsRef<Path>,
        start: usize,
    ) -> std::io::Result<()> {
        let mut file = std::fs::File::open(path)?;
        file.read(&mut self.mem[start..])?;
        Ok(())
    }

    /// Loads an Intel HEX file into memory.
    fn load_program_intel_hex(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;

        for line in content.lines() {
            if !line.starts_with(':') {
                continue;
            }

            let byte_count = u8::from_str_radix(&line[1..3], 16).unwrap();
            let address = u16::from_str_radix(&line[3..7], 16).unwrap();
            let record_type = u8::from_str_radix(&line[7..9], 16).unwrap();

            if record_type == 0x00 {
                for i in 0..byte_count {
                    let offset = 9 + (i as usize * 2);
                    let byte = u8::from_str_radix(&line[offset..offset + 2], 16).unwrap();
                    self.mem[(address + i as u16) as usize] = byte;
                }
            } else if record_type == 0x01 {
                break;
            }
        }
        Ok(())
    }
}

impl Bus for KlausDormann6502 {
    // Read and write need to be implemented as memory-mapped I/O.
    fn read(&self, addr: u16) -> u8 {
        if addr == 0xf0_04 {
            // You could return an uppercase C to auto-continue, since the
            // options are continue and skip but for now I'll do that manually.
            // b'C'
            let mut input = String::with_capacity(1);
            std::io::stdin().read_line(&mut input).unwrap_or(0);
            input.as_bytes()[0]
        } else {
            self.mem[addr as usize]
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr == 0xf0_01 {
            println!("{}", data as char);
        } else {
            self.mem[addr as usize] = data;
        }
    }

    fn check_dma_request(&self) -> bool {
        false
    }

    fn check_irq(&self) -> bool {
        self.mem[INTERRUPT_TEST_FEEDBACK] & INTERRUPT_TEST_IRQ != 0
    }

    fn check_nmi(&self) -> bool {
        self.mem[INTERRUPT_TEST_FEEDBACK] & INTERRUPT_TEST_NMI != 0
    }

    fn check_reset(&self) -> bool {
        false
    }

    fn cycle(&mut self) {}

    fn get_active_dma_channel(&self) -> Option<u8> {
        None
    }

    fn perform_dma_transfer(&self, _channel: u8) -> crate::DMATransferStatus {
        crate::DMATransferStatus::Complete
    }
}
