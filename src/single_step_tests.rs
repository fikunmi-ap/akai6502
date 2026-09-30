//! Validation against the SingleStepTests 6502 reference vectors.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::Bus;
use crate::CPU;
use crate::cpu::CPUState;
use crate::opcodes::Opcode;
use crate::test_bus::{BusOperation, TestBus};

const DOCUMENTED_OPCODE_COUNT: usize = 151;
const TEST_DIRECTORY_ENV: &str = "SINGLE_STEP_TESTS_DIR";
const OPCODE_ENV: &str = "SINGLE_STEP_OPCODE";

#[derive(Debug, Deserialize)]
struct ReferenceTest {
    name: String,
    initial: ReferenceState,
    #[serde(rename = "final")]
    final_state: ReferenceState,
    cycles: Vec<(u16, u8, CycleKind)>,
}

#[derive(Debug, Deserialize)]
struct ReferenceState {
    pc: u16,
    /// Stack pointer, conventionally named S in 6502 documentation.
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    /// Processor status register, conventionally named P.
    p: u8,
    ram: Vec<(u16, u8)>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CycleKind {
    Read,
    Write,
}

#[derive(Debug, PartialEq, Eq)]
struct ProcessorState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
}

impl ProcessorState {
    fn from_cpu(cpu: &CPU<TestBus>) -> Self {
        Self {
            pc: cpu.pc,
            s: cpu.sp,
            a: cpu.a,
            x: cpu.x,
            y: cpu.y,
            p: cpu.sr,
        }
    }

    fn from_reference(state: &ReferenceState) -> Self {
        Self {
            pc: state.pc,
            s: state.s,
            a: state.a,
            x: state.x,
            y: state.y,
            p: state.p,
        }
    }
}

fn test_directory() -> PathBuf {
    std::env::var_os(TEST_DIRECTORY_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/65x02/6502/v1"))
}

fn documented_opcodes() -> Vec<u8> {
    let opcodes: Vec<_> = (0..=u8::MAX)
        .filter(|opcode| Opcode::from_byte(*opcode) != Opcode::Unsupported)
        .collect();

    assert_eq!(opcodes.len(), DOCUMENTED_OPCODE_COUNT);
    opcodes
}

fn selected_opcodes() -> Vec<u8> {
    let Some(opcode) = std::env::var_os(OPCODE_ENV) else {
        return documented_opcodes();
    };
    let opcode = opcode
        .to_str()
        .unwrap_or_else(|| panic!("{OPCODE_ENV} must be valid UTF-8"));
    let opcode = u8::from_str_radix(opcode.trim_start_matches("0x"), 16)
        .unwrap_or_else(|_| panic!("{OPCODE_ENV} must be a hexadecimal byte"));

    assert_ne!(
        Opcode::from_byte(opcode),
        Opcode::Unsupported,
        "${opcode:02X} is not a documented NMOS 6502 opcode"
    );
    vec![opcode]
}

fn load_reference_tests(opcode: u8) -> Vec<ReferenceTest> {
    let path = test_directory().join(format!("{opcode:02x}.json"));
    let json = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read {}: {error}\n\
             see the single-step test setup in README.md",
            path.display()
        )
    });

    serde_json::from_str(&json)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn set_initial_state(cpu: &mut CPU<TestBus>, state: &ReferenceState) {
    cpu.pc = state.pc;
    cpu.sp = state.s;
    cpu.a = state.a;
    cpu.x = state.x;
    cpu.y = state.y;
    cpu.sr = state.p;
    cpu.cycles = 0;
    cpu.state = CPUState::FetchOpcode;
}

fn expected_operations(test: &ReferenceTest) -> Vec<BusOperation> {
    test.cycles
        .iter()
        .map(|&(address, data, kind)| match kind {
            CycleKind::Read => BusOperation::Read { address, data },
            CycleKind::Write => BusOperation::Write { address, data },
        })
        .collect()
}

fn run_reference_test(test: &ReferenceTest) {
    let mut cpu = CPU::new();
    let mut bus = TestBus::new();

    set_initial_state(&mut cpu, &test.initial);
    for &(address, data) in &test.initial.ram {
        bus.write(address, data);
    }
    bus.clear_bus_operations();

    cpu.execute_single_instruction(&mut bus);

    let actual_operations = bus.bus_operations();
    let expected_operations = expected_operations(test);
    assert_eq!(
        actual_operations, expected_operations,
        "bus-cycle mismatch in reference case {}",
        test.name
    );
    assert_eq!(
        cpu.cycles,
        test.cycles.len() as u64,
        "cycle-count mismatch in reference case {}",
        test.name
    );
    assert_eq!(
        ProcessorState::from_cpu(&cpu),
        ProcessorState::from_reference(&test.final_state),
        "processor-state mismatch in reference case {}",
        test.name
    );

    for &(address, expected) in &test.final_state.ram {
        let actual = bus.read(address);
        assert_eq!(
            actual, expected,
            "memory mismatch at ${address:04X} in reference case {}",
            test.name
        );
    }
}

#[test]
#[ignore = "requires the external SingleStepTests/65x02 test vectors"]
fn test_nmos_6502_single_step_reference() {
    for opcode in selected_opcodes() {
        let tests = load_reference_tests(opcode);
        assert_eq!(
            tests.len(),
            10_000,
            "expected 10,000 reference cases for opcode ${opcode:02X}"
        );

        for test in &tests {
            run_reference_test(test);
        }
    }
}
