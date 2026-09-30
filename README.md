# akai6502

`akai6502` is a cycle-accurate NMOS 6502 emulator written in Rust. It
implements all 151 documented opcodes, including decimal arithmetic,
interrupt timing, dummy bus accesses, page-crossing behavior, and the
original indirect `JMP` page-wrap bug.

The long-term goal is to build a cycle-accurate, high-performance core that
can be used as the foundation for emulating 6502-based systems, starting with
the NES.

Version 0.1 covers the documented NMOS 6502 instruction set. Expect breaking
changes as the project develops.

## Why Build This at All?

The 6502 is nearly fifty years old, but a lot of the things it did are still
done today in one form or another. Implementing an emulator for it teaches a
level of mechanical sympathy I've found hard to get elsewhere. And unlike
more recent CPUs, a cycle-accurate 6502 is tractable for one person in their
spare time.

## Features

- All 151 documented NMOS 6502 opcodes and addressing modes.
- Cycle-by-cycle bus reads and writes, including dummy accesses.
- NMOS decimal-mode behavior for `ADC` and `SBC`.
- Cycle-accurate `RESET`, `NMI`, and `IRQ` entry.
- NMOS interrupt-disable timing around `CLI`, `SEI`, `PLP`, and `RTI`.
- Page-crossing penalties, indexed-store cycles, and read-modify-write dummy
  writes.
- The NMOS indirect `JMP` page-wrap bug.
- No runtime dependencies.

## Usage

The crate is not on crates.io yet. It can be used directly from this
repository:

```toml
[dependencies]
akai6502 = { git = "https://github.com/fikunmi-ap/akai6502", tag = "v0.1.0" }
```

The CPU communicates with the rest of the emulated machine through the
`Bus` trait. The included `TestBus` provides a simple 64 KiB memory for trying
the CPU without first implementing a complete system bus.

```rust
use akai6502::{Bus, CPU, test_bus::TestBus};

fn main() {
    let mut memory = TestBus::new();

    // LDA #$42; STA $10
    memory.write(0x8000, 0xa9);
    memory.write(0x8001, 0x42);
    memory.write(0x8002, 0x85);
    memory.write(0x8003, 0x10);

    let mut cpu = CPU::<TestBus>::new();
    cpu.set_pc(0x8000);
    cpu.execute_single_instruction(&mut memory);
    cpu.execute_single_instruction(&mut memory);

    assert_eq!(memory.read(0x0010), 0x42);
    assert_eq!(cpu.pc(), 0x8004);
    assert_eq!(cpu.cycles(), 5);
}
```

For a complete emulator, implement `Bus` to map reads and writes to RAM, ROM,
and I/O devices. The bus's `cycle` method is called at the end of every CPU
cycle so that those devices can advance in step with it. Interrupt lines are
exposed through `check_nmi`, `check_irq`, and `check_reset`.

## Architecture

The design has two main pieces: the `CPU` and the `Bus`.

`CPU<B>` is generic over a type that implements `Bus`. The CPU owns the
registers and instruction machinery, while the bus owns the address space and
everything connected to it. This keeps the CPU independent of the machine's
memory map and peripherals. It does not need to know whether an address points
to RAM, ROM, or an I/O device; that decision belongs to the machine using the
core.

Using a generic means bus calls are statically dispatched instead of going
through a trait object on every cycle.

An instruction is executed in three broad phases:

1. Fetch the opcode.
2. Resolve the operand address.
3. Execute the operation.

The 6502 uses fixed-width (8-bit) opcodes, so there are only 256 possible
values and the documented opcodes occupy ~59% of the opcode space, so
it is somewhat dense. These properties make a jump-table approach attractive.

The opcode indexes the table whose entries pair an addressing mode with an
operation. This lets instructions share addressing-mode logic.
Unfortunately, the split in the actual machine is not that clean. A few
instructions need their bus operations to happen in a different order.
Branches, stack instructions, indexed stores, read-modify-write instructions,
and interrupts use their own bus sequences where the hardware does something
different. It is likely that future versions will move away from the
addressing-mode/operation split for that reason.

Cycle accuracy lives at the bus-access boundary. Every read or write consumes
one CPU cycle, then `Bus::cycle` is called so the rest of the machine can move
forward. Dummy reads and writes are performed as real bus operations instead
of simply adding to a counter. The public API still executes a complete
instruction at a time, but the bus sees every cycle in the correct order.

Interrupt and reset lines also come from the bus. The CPU samples `RESET`,
`NMI`, and `IRQ` during execution, then services them at instruction boundaries
with cycle-accurate entry sequences. `RESET` has the highest priority, followed
by `NMI`, then `IRQ` when the interrupt-disable flag allows it.

## Accuracy and Testing

Run the normal test suite with:

```sh
cargo test --release
```

No additional setup is needed for this command. It runs the unit and bus-trace
tests, the Klaus Dormann functional and interrupt tests, and Bruce Clark's
exhaustive decimal-mode test. Their test programs are included in the
repository.

The optional single-step suite requires a separate download. Its ignored test
compares the final CPU and memory state, along with every bus cycle, against
the [65x02 test suite](https://github.com/SingleStepTests/65x02). The test
targets revision `2f6980a2d95757486c7bee24355c360e40e2a224`.
All 10,000 cases pass for each of the 151 documented opcode encodings at that
revision.

Clone the reference data into the default ignored location:

```sh
git clone --filter=blob:none --no-checkout \
    https://github.com/SingleStepTests/65x02.git tests/65x02
git -C tests/65x02 sparse-checkout set 6502/v1
git -C tests/65x02 fetch --depth 1 origin \
    2f6980a2d95757486c7bee24355c360e40e2a224
git -C tests/65x02 checkout --detach FETCH_HEAD
```

Run all 10,000 cases for each documented opcode:

```sh
cargo test --release single_step -- --ignored
```

To run a single opcode while debugging the harness, pass its hexadecimal byte:

```sh
SINGLE_STEP_OPCODE=ea cargo test --release single_step -- --ignored
```

Set `SINGLE_STEP_TESTS_DIR` if the `6502/v1` directory is stored somewhere
else.

## Current Limitations

- Undocumented opcodes are not implemented.
- The public execution API advances one complete instruction at a time. Bus
  operations are cycle-accurate, but the CPU cannot yet be paused and resumed
  in the middle of an instruction.
- The DMA interface exists, but CPU handoff and cycle stealing are not
  implemented.
- Debug builds currently print the CPU state after every instruction. Release
  builds do not.
- This emulates an NMOS 6502, not the Ricoh 2A03 and 2A07 CPUs used by the
  NES. NES-specific CPU behavior and the rest of the console are planned
  separately.

## Resources

- [Masswerk 6502 instruction set][masswerk]
- [Obelisk 6502 reference][obelisk]
- [MOS MCS6500 programming manual][mos]

[masswerk]: https://www.masswerk.at/6502/6502_instruction_set.html
[obelisk]: https://www.nesdev.org/obelisk-6502-guide/reference.html
[mos]: https://archive.org/details/mos_microcomputers_programming_manual

## License

The emulator is available under either the MIT License or the Apache License
2.0, at your option.

The bundled test programs retain their original licenses. See
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for details.
