//! Instructions


use crate::{
    addressing::AddressingMode, cpu::CPU, Bus, Word, BREAK_FLAG, CARRY_FLAG, DECIMAL_FLAG, INTERRUPT_FLAG, NEGATIVE_FLAG, OVERFLOW_FLAG, UNUSED_FLAG, ZERO_FLAG
};

/// Creates the opcode jump table.
///
/// It is expected that this function is called once per program; at start-up.
/// It is not baked directly into the binary to accommodate the use of generic
/// parameters.
///
/// TODO: Use a macro to make the code slightly more readable e.g:
/// macro_rules! instr_table_entry {
///    () => {
///        
///    };
/// }
pub fn create_opcode_table<B: Bus>() -> [InstructionHandler<B>; 256] {
    let mut table: [InstructionHandler<B>; 256] = 
        core::array::from_fn(|_| InstructionHandler::default());
    
    // Row 0x0_
    table[0x00] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: brk };
    table[0x01] = InstructionHandler { addr_mode: AddressingMode::IndexedIndirect, operation: ora };
    table[0x05] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: ora };
    table[0x06] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: asl };
    table[0x08] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: php };
    table[0x09] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: ora };
    table[0x0a] = InstructionHandler { addr_mode: AddressingMode::Accumulator, operation: asl };
    table[0x0d] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: ora };
    table[0x0e] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: asl };

    // Row 0x1_
    table[0x10] = InstructionHandler { addr_mode: AddressingMode::Relative, operation: bpl };
    table[0x11] = InstructionHandler { addr_mode: AddressingMode::IndirectIndexed, operation: ora };
    table[0x15] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: ora };
    table[0x16] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: asl };
    table[0x18] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: clc };
    table[0x19] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: ora };
    table[0x1d] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: ora };
    table[0x1e] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: asl };

    // Row 0x2_
    table[0x20] = InstructionHandler { addr_mode: AddressingMode::AbsoluteJsr, operation: jsr };
    table[0x21] = InstructionHandler { addr_mode: AddressingMode::IndexedIndirect, operation: and };
    table[0x24] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: bit };
    table[0x25] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: and };
    table[0x26] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: rol };
    table[0x28] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: plp };
    table[0x29] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: and };
    table[0x2a] = InstructionHandler { addr_mode: AddressingMode::Accumulator, operation: rol };
    table[0x2c] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: bit };
    table[0x2d] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: and };
    table[0x2e] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: rol };

    // Row 0x3_
    table[0x30] = InstructionHandler { addr_mode: AddressingMode::Relative, operation: bmi };
    table[0x31] = InstructionHandler { addr_mode: AddressingMode::IndirectIndexed, operation: and };
    table[0x35] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: and };
    table[0x36] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: rol };
    table[0x38] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: sec };
    table[0x39] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: and };
    table[0x3d] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: and };
    table[0x3e] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: rol };

    // Row 0x4_
    table[0x40] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: rti };
    table[0x41] = InstructionHandler { addr_mode: AddressingMode::IndexedIndirect, operation: eor };
    table[0x45] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: eor };
    table[0x46] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: lsr };
    table[0x48] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: pha };
    table[0x49] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: eor };
    table[0x4a] = InstructionHandler { addr_mode: AddressingMode::Accumulator, operation: lsr };
    table[0x4c] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: jmp };
    table[0x4d] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: eor };
    table[0x4e] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: lsr };

    // Row 0x5_
    table[0x50] = InstructionHandler { addr_mode: AddressingMode::Relative, operation: bvc };
    table[0x51] = InstructionHandler { addr_mode: AddressingMode::IndirectIndexed, operation: eor };
    table[0x55] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: eor };
    table[0x56] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: lsr };
    table[0x58] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: cli };
    table[0x59] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: eor };
    table[0x5d] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: eor };
    table[0x5e] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: lsr };

    // Row 0x6_
    table[0x60] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: rts };
    table[0x61] = InstructionHandler { addr_mode: AddressingMode::IndexedIndirect, operation: adc };
    table[0x65] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: adc };
    table[0x66] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: ror };
    table[0x68] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: pla };
    table[0x69] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: adc };
    table[0x6a] = InstructionHandler { addr_mode: AddressingMode::Accumulator, operation: ror };
    table[0x6c] = InstructionHandler { addr_mode: AddressingMode::Indirect, operation: jmp };
    table[0x6d] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: adc };
    table[0x6e] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: ror };

    // Row 0x7_
    table[0x70] = InstructionHandler { addr_mode: AddressingMode::Relative, operation: bvs };
    table[0x71] = InstructionHandler { addr_mode: AddressingMode::IndirectIndexed, operation: adc };
    table[0x75] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: adc };
    table[0x76] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: ror };
    table[0x78] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: sei };
    table[0x79] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: adc };
    table[0x7d] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: adc };
    table[0x7e] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: ror };

    // Row 0x8_
    table[0x81] = InstructionHandler { addr_mode: AddressingMode::IndexedIndirect, operation: sta };
    table[0x84] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: sty };
    table[0x85] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: sta };
    table[0x86] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: stx };
    table[0x88] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: dey };
    table[0x8a] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: txa };
    table[0x8c] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: sty };
    table[0x8d] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: sta };
    table[0x8e] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: stx };

    // Row 0x9_
    table[0x90] = InstructionHandler { addr_mode: AddressingMode::Relative, operation: bcc };
    table[0x91] = InstructionHandler { addr_mode: AddressingMode::IndirectIndexed, operation: sta };
    table[0x94] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: sty };
    table[0x95] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: sta };
    table[0x96] = InstructionHandler { addr_mode: AddressingMode::ZeroPageY, operation: stx };
    table[0x98] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: tya };
    table[0x99] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: sta };
    table[0x9a] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: txs };
    table[0x9d] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: sta };

    // Row 0xa_
    table[0xa0] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: ldy };
    table[0xa1] = InstructionHandler { addr_mode: AddressingMode::IndexedIndirect, operation: lda };
    table[0xa2] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: ldx };
    table[0xa4] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: ldy };
    table[0xa5] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: lda };
    table[0xa6] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: ldx };
    table[0xa8] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: tay };
    table[0xa9] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: lda };
    table[0xaa] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: tax };
    table[0xac] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: ldy };
    table[0xad] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: lda };
    table[0xae] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: ldx };

    // Row 0xb_
    table[0xb0] = InstructionHandler { addr_mode: AddressingMode::Relative, operation: bcs };
    table[0xb1] = InstructionHandler { addr_mode: AddressingMode::IndirectIndexed, operation: lda };
    table[0xb4] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: ldy };
    table[0xb5] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: lda };
    table[0xb6] = InstructionHandler { addr_mode: AddressingMode::ZeroPageY, operation: ldx };
    table[0xb8] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: clv };
    table[0xb9] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: lda };
    table[0xba] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: tsx };
    table[0xbc] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: ldy };
    table[0xbd] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: lda };
    table[0xbe] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: ldx };

    // Row 0xc_
    table[0xc0] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: cpy };
    table[0xc1] = InstructionHandler { addr_mode: AddressingMode::IndexedIndirect, operation: cmp };
    table[0xc4] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: cpy };
    table[0xc5] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: cmp };
    table[0xc6] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: dec };
    table[0xc8] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: iny };
    table[0xc9] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: cmp };
    table[0xca] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: dex };
    table[0xcc] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: cpy };
    table[0xcd] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: cmp };
    table[0xce] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: dec };

    // Row 0xd_
    table[0xd0] = InstructionHandler { addr_mode: AddressingMode::Relative, operation: bne };
    table[0xd1] = InstructionHandler { addr_mode: AddressingMode::IndirectIndexed, operation: cmp };
    table[0xd5] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: cmp };
    table[0xd6] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: dec };
    table[0xd8] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: cld };
    table[0xd9] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: cmp };
    table[0xdd] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: cmp };
    table[0xde] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: dec };

    // Row 0xe_
    table[0xe0] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: cpx };
    table[0xe1] = InstructionHandler { addr_mode: AddressingMode::IndexedIndirect, operation: sbc };
    table[0xe4] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: cpx };
    table[0xe5] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: sbc };
    table[0xe6] = InstructionHandler { addr_mode: AddressingMode::ZeroPage, operation: inc };
    table[0xe8] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: inx };
    table[0xe9] = InstructionHandler { addr_mode: AddressingMode::Immediate, operation: sbc };
    table[0xea] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: nop };
    table[0xec] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: cpx };
    table[0xed] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: sbc };
    table[0xee] = InstructionHandler { addr_mode: AddressingMode::Absolute, operation: inc };

    // Row 0xf_
    table[0xf0] = InstructionHandler { addr_mode: AddressingMode::Relative, operation: beq };
    table[0xf1] = InstructionHandler { addr_mode: AddressingMode::IndirectIndexed, operation: sbc };
    table[0xf5] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: sbc };
    table[0xf6] = InstructionHandler { addr_mode: AddressingMode::ZeroPageX, operation: inc };
    table[0xf8] = InstructionHandler { addr_mode: AddressingMode::Implied, operation: sed };
    table[0xf9] = InstructionHandler { addr_mode: AddressingMode::AbsoluteY, operation: sbc };
    table[0xfd] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: sbc };
    table[0xfe] = InstructionHandler { addr_mode: AddressingMode::AbsoluteX, operation: inc };

    table
}

#[derive(Debug)]
pub(crate) struct InstructionHandler<B: Bus> {
    pub addr_mode: AddressingMode,
    pub operation: fn(&mut CPU<B>, &mut B),
}

fn unsupported_instructions<B: Bus>(_cpu: &mut CPU<B>, _bus: &mut B) {}

impl<B: Bus> Default for InstructionHandler<B> {
    fn default() -> Self {
        InstructionHandler {
            addr_mode: AddressingMode::Implied,
            operation: unsupported_instructions,
        }
    }
}

/// Check if the current addressing mode could incur a page cross penalty and
/// if it does, then perform the operand read. 
macro_rules! extra_cycle {
    ($cpu: ident, $bus: ident) => {
        if $cpu.extra_cycle {
            $cpu.fetch_byte($cpu.addr_oper_prov, $bus);
        }
    };
}

/// Branch Instruction Macro
/// 
/// This macro implements the cycle-accurate behavior of 6502 branch
/// instructions.
/// 
/// Branch instructions take 2 cycles when not taken, 3 cycles when taken to
/// the same page, and 4 cycles when taken across a page boundary.
/// 
/// # Parameters
/// - `$condition`: A boolean expression that determines if the branch should
///    be taken.
/// - `$cpu`: Mutable reference to the CPU instance
/// - `$bus`: Mutable reference to the Bus instance
/// 
/// # Behavior
/// 1. **Cycle 1**: Fetch the signed 8-bit relative offset operand
/// 2. **Branch Not Taken**: If condition is false, return immediately (2
///                          cycles total)
/// 3. **Branch Taken**: Read and discard the byte at the next sequential PC
/// 4. **Page Cross**: Read and discard from the destination with the old high
///                    byte before correcting it
/// 5. Update the program counter to the final destination
///
/// # Hardware Simulation Details
/// A taken branch has already started reading the next sequential instruction
/// by the time its condition and offset are available. It discards that read
/// and applies the offset to the low byte of the PC. If that addition crosses
/// a page, it exposes one more read using the old high byte before correcting
/// the address.
///
/// # Usage
/// Used by all conditional branch instructions: BCC, BCS, BEQ, BMI, BNE, BPL,
/// BVC, BVS
macro_rules! branch {
    ($condition: expr, $cpu: ident, $bus: ident) => {
        $cpu.fetch_operand($bus);
        if !$condition {
            // branch not taken.
            return
        }
        let offset = $cpu.operand as i8;
        let pc = $cpu.pc;
        let dest = pc.wrapping_add(offset as Word);
        $cpu.dummy_read($bus);
        if dest & 0xff_00 != pc & 0xff_00 {
            let dest_prov = (pc & 0xff_00) | (dest & 0x00_ff);
            $cpu.fetch_byte(dest_prov, $bus);
        }
        $cpu.pc = dest;
    };
}

/// RMW operations perform a unique read-write-write sequence.
///
/// The first write simply writes the value it just read back to the target
/// address. The read and the first write are common to all RMW operations.
fn rmw_read_write<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // In absolute-X addressing mode, there is an extra cycle spent reading the
    // provisional address. If a page is crossed, only the second read uses the
    // correct address. If no page is crossed (no carry after adding X to the
    // low byte of the address), the same address is read twice.
    if cpu.addr_mode == AddressingMode::AbsoluteX {
        cpu.fetch_byte(cpu.addr_oper_prov, bus);
    }
    cpu.fetch_operand(bus);
    cpu.write_byte(cpu.addr_oper, cpu.operand, bus);
}

/// Name: Add with Carry (ADC)
/// Function: A = A + M + C
/// Flags Updated: C, Z, V, N.
/// 
/// ADC is a simple operation. The only bit of complexity is in setting the
/// overflow flag update condition.
/// 
/// An overflow occurs whenever the computed value does not fit into the range
/// for signed addition (in this case -128 to +127).
/// 
/// An overflow has occurred when:
/// - adding two positives returns a negative.
/// - adding two negatives returns a positive.
/// 
/// Mixed sums are always safe.
/// 
/// Whenever the operands have the same sign and the result has a different
/// sign (from the operands), then there is an overflow. That can be expressed
/// as:
/// ```pseudocode
///     V <- (!(A ^ M) & (A ^ R) & 0x80) != 0
/// ```
/// 
/// Which can be read as if a and m are the same (not xor) and a and r are not
/// the same (xor) then there's an overflow.
/// 
/// # Decimal Mode
/// In decimal mode, each nibble (4-bits) is treated as a decimal digit as in
/// Binary-Coded Decimal (BCD).
/// 
/// BCD addition uses the same circuits as normal addition with a few
/// modifications:
/// 
/// 1. Addition is performed on nibbles (4 bits) not words.
/// 2. Each resulting nibble is checked for validity; if the sum is greater
/// than 9 (0b1001), the value must be corrected by adding 6.
/// 
/// This is done per nibble. And in the event that the high nibble is > 9 then
/// you add 6 to it as well (0x60 to the whole word) and propagate the carry to
/// the next nibble.
/// 
/// On the NMOS 6502, the decimal-mode flags come from different stages of the
/// addition. Z comes from the binary result, N and V come from the intermediate
/// result after the low-nibble correction, and C comes from the final decimal
/// correction. They are deterministic but do not describe the final BCD result.
fn adc<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);

    let old_a = cpu.a;
    let operand = cpu.operand;
    let carry = cpu.get_flag(CARRY_FLAG);
    let res = old_a as Word + operand as Word + carry as Word;

    if cpu.get_flag(DECIMAL_FLAG) {
        let mut low_nibble =
            (old_a & 0x0f) + (operand & 0x0f) + carry as u8;
        let low_carry = low_nibble > 9;

        if low_carry {
            low_nibble = (low_nibble - 10) & 0x0f;
        }

        let mut high_nibble =
            (old_a >> 4) + (operand >> 4) + low_carry as u8;
        let negative = high_nibble & 0x08 != 0;
        let carry_out = high_nibble > 9;

        if carry_out {
            high_nibble = (high_nibble - 10) & 0x0f;
        }

        cpu.a = (high_nibble << 4) | low_nibble;

        let overflow = ((old_a >= 0x80) ^ negative)
            && ((operand >= 0x80) ^ negative);

        cpu.set_flag(NEGATIVE_FLAG, negative);
        cpu.set_flag(OVERFLOW_FLAG, overflow);
        cpu.set_flag(ZERO_FLAG, res & 0xff == 0);
        cpu.set_flag(CARRY_FLAG, carry_out);
    } else {
        cpu.a = (res & 0xff) as u8;
        cpu.set_flag(CARRY_FLAG, res > 0xff);
        cpu.set_flag(NEGATIVE_FLAG, cpu.a & 0x80 != 0);
        cpu.set_flag(ZERO_FLAG, cpu.a == 0);
        cpu.set_flag(
            OVERFLOW_FLAG,
            ((old_a as Word ^ res) & !(old_a as Word ^ operand as Word))
                & 0x80
                != 0,
        );
    }
}

/// Name: Logical AND (AND)
/// Function: A = A & M
/// Flags Updated: N, Z
/// 
/// AND performs a bitwise logical AND operation between the accumulator and
/// the operand from memory. Each bit of the result is 1 only if the
/// corresponding bits in both operands are 1.
/// 
/// The operation is straightforward:
/// - For each bit position i: result[i] = A[i] & M[i]
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
fn and<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);
    cpu.a = cpu.a & cpu.operand;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.a & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.a == 0);
}

/// Name: Arithmetic Shift Left (ASL)
/// Function: C <- [76543210] <- 0
/// Flags Updated: N, Z, C
/// 
/// ASL shifts all bits of the operand one position to the left. Bit 0 is
/// filled with 0, and the original bit 7 is shifted into the carry flag.
/// 
/// This is a Read-Modify-Write (RMW) instruction that can operate on either:
/// - The accumulator (implied addressing)
/// - A memory location (various addressing modes)
/// 
/// The operation effectively multiplies the operand by 2 (ignoring overflow).
/// 
/// # RMW Instruction Timing
/// RMW instructions have special timing characteristics:
/// 1. They read the original value
/// 2. Write the original value back (dummy write)
/// 3. Write the modified value
/// 
/// For absolute-X mode (ABX), an additional cycle is consumed
/// because the CPU doesn't pipeline the effective address calculation.
fn asl<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    // TODO: micro-optimization: if we were handling fetch here, cpu.operand
    // would be used as an internal register but we're not so we have to work
    // around that for now.
    let temp = if cpu.addr_mode == AddressingMode::Accumulator {
        cpu.dummy_read(bus);
        let temp = (cpu.a as u16) << 1;
        cpu.a = (temp & 0xff) as u8;
        temp
    } else {
        rmw_read_write(cpu, bus);
        let temp = (cpu.operand as u16) << 1;
        cpu.write_byte(cpu.addr_oper, (temp & 0xff) as u8, bus);
        temp
    };
    cpu.set_flag(NEGATIVE_FLAG, (temp & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, temp & 0xff == 0);
    cpu.set_flag(CARRY_FLAG, temp > 0xff);
}

/// Name: Branch if Carry Clear (BCC)
/// Function: Branch if C = 0
/// Flags Updated: None
/// 
/// BCC tests the carry flag and branches if it is clear (0). The branch
/// target is calculated by adding the signed 8-bit operand to the program
/// counter.
/// 
/// # Branch Timing
/// Branch instructions have variable timing:
/// - 2 cycles if branch is not taken
/// - 3 cycles if branch is taken and stays on same page
/// - 4 cycles if branch is taken and crosses page boundary
/// 
/// The 6502 can only perform 8-bit arithmetic, so page boundary crossings
/// require an additional cycle to fix up the high byte of the address.
fn bcc<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    branch!(!cpu.get_flag(CARRY_FLAG), cpu, bus);
}

/// Name: Branch if Carry Set (BCS)
/// Function: Branch if C = 1
/// Flags Updated: None
/// 
/// BCS tests the carry flag and branches if it is set (1). The branch
/// target is calculated by adding the signed 8-bit operand to the program
/// counter.
/// 
/// See BCC for branch timing details.
fn bcs<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    branch!(cpu.get_flag(CARRY_FLAG), cpu, bus);
}

/// Name: Branch if Equal (BEQ)
/// Function: Branch if Z = 1
/// Flags Updated: None
/// 
/// BEQ tests the zero flag and branches if it is set (1). This is typically
/// used after a comparison instruction to branch if two values are equal.
/// 
/// See BCC for branch timing details.
fn beq<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    branch!(cpu.get_flag(ZERO_FLAG), cpu, bus);
}

/// Name: Bit Test (BIT)
/// Function: Test bits in memory with accumulator
/// Flags Updated: N, V, Z
/// 
/// BIT performs a logical AND between the accumulator and the operand, but
/// unlike AND, it doesn't store the result. Instead, it only updates flags.
/// 
/// Flag updates are unique for BIT:
/// - N (Negative): Set to bit 7 of the operand (not the result)
/// - V (Overflow): Set to bit 6 of the operand (not the result)
/// - Z (Zero): Set if (A & M) == 0
/// 
/// This instruction is commonly used to test specific bits in memory or
/// I/O registers without modifying the accumulator.
fn bit<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.fetch_operand(bus);
    cpu.set_flag(NEGATIVE_FLAG, (cpu.operand & 0x80) != 0);
    cpu.set_flag(OVERFLOW_FLAG, (cpu.operand & 0x40) != 0);
    cpu.set_flag(ZERO_FLAG, (cpu.operand & cpu.a) == 0);
}

/// Name: Branch if Minus (BMI)
/// Function: Branch if N = 1
/// Flags Updated: None
/// 
/// BMI tests the negative flag and branches if it is set (1). This is
/// typically used after an operation to branch if the result was negative
/// (bit 7 set).
/// 
/// See BCC for branch timing details.
fn bmi<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    branch!(cpu.get_flag(NEGATIVE_FLAG), cpu, bus);
}

/// Name: Branch if Not Equal (BNE)
/// Function: Branch if Z = 0
/// Flags Updated: None
/// 
/// BNE tests the zero flag and branches if it is clear (0). This is typically
/// used after a comparison instruction to branch if two values are not equal.
/// 
/// See BCC for branch timing details.
fn bne<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    branch!(!cpu.get_flag(ZERO_FLAG), cpu, bus);
}

/// Name: Branch if Plus (BPL)
/// Function: Branch if N = 0
/// Flags Updated: None
/// 
/// BPL tests the negative flag and branches if it is clear (0). This is
/// typically used after an operation to branch if the result was positive
/// (bit 7 clear).
/// 
/// See BCC for branch timing details.
fn bpl<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    branch!(!cpu.get_flag(NEGATIVE_FLAG), cpu, bus);
}

/// Name: Break (BRK)
/// Function: Software interrupt
/// Flags Updated: B, I
/// 
/// BRK is a software interrupt similar in function to a hardware interrupt.
/// It pushes the program counter and status register to the stack, then
/// jumps to the interrupt vector at $fffe-$ffff.
/// 
/// The sequence of operations is:
/// 1. Fetch the BRK opcode
/// 2. Increment PC (dummy fetch of next byte)
/// 3. Push PCH to stack
/// 4. Push PCL to stack  
/// 5. Push status register with B flag set to stack
/// 6. Set I flag to disable interrupts
/// 7. Load PC from interrupt vector ($fffe-$ffff)
/// 
/// The return address pushed to the stack is PC + 2 (from the initial BRK
/// instruction), providing space for a "break mark" byte that can be used by
/// the interrupt handler to distinguish between different software interrupts.
/// 
/// # Break Flag Behavior
/// The B flag is set in the status register copy pushed to the stack, but
/// the actual status register's B flag behavior is implementation-dependent.
/// The RTI instruction will restore the original B flag state.
fn brk<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    // dummy fetch to match IRQ timing.
    let _padding = cpu.fetch_from_and_increment_pc(bus);
    cpu.push_word_to_stack(bus, cpu.pc);
    cpu.push_byte_to_stack(bus, cpu.sr | BREAK_FLAG | UNUSED_FLAG);
    cpu.set_flag(INTERRUPT_FLAG, true);
    cpu.refresh_irq_poll_i_flag();
    let pc_lo = cpu.fetch_byte(0xff_fe, bus);
    let pc_hi = cpu.fetch_byte(0xff_ff, bus);
    cpu.pc = (pc_hi as Word) << 8 | pc_lo as Word;
}

/// Name: Branch if Overflow Clear (BVC)
/// Function: Branch if V = 0
/// Flags Updated: None
/// 
/// BVC tests the overflow flag and branches if it is clear (0). This is
/// typically used after arithmetic operations to branch if no signed
/// overflow occurred.
/// 
/// See BCC for branch timing details.
fn bvc<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    branch!(!cpu.get_flag(OVERFLOW_FLAG), cpu, bus);
}

/// Name: Branch if Overflow Set (BVS)
/// Function: Branch if V = 1
/// Flags Updated: None
/// 
/// BVS tests the overflow flag and branches if it is set (1). This is
/// typically used after arithmetic operations to branch if signed
/// overflow occurred.
/// 
/// See BCC for branch timing details.
fn bvs<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    branch!(cpu.get_flag(OVERFLOW_FLAG), cpu, bus);
}

/// Name: Clear Carry Flag (CLC)
/// Function: C = 0
/// Flags Updated: C
/// 
/// CLC clears the carry flag to 0. This is commonly used before addition
/// operations when you don't want the previous carry to affect the result,
/// or to clear the carry flag for use as a general-purpose flag.
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, CLC performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn clc<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.dummy_read(bus);
    cpu.set_flag(CARRY_FLAG, false);
}

/// Name: Clear Decimal Flag (CLD)
/// Function: D = 0
/// Flags Updated: D
/// 
/// CLD clears the decimal flag to 0, switching the processor from Binary
/// Coded Decimal (BCD) mode to normal binary arithmetic mode.
/// 
/// In binary mode, ADC and SBC perform standard binary arithmetic.
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, CLD performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn cld<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.dummy_read(bus);
    cpu.set_flag(DECIMAL_FLAG, false);
}

/// Name: Clear Interrupt Disable Flag (CLI)
/// Function: I = 0
/// Flags Updated: I
/// 
/// CLI clears the interrupt disable flag to 0, enabling maskable interrupts
/// (IRQ).
/// 
/// When the I flag is clear, the processor will respond to IRQ signals.
/// When set, IRQ interrupts are ignored (but NMI interrupts are still
/// processed).
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, CLI performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn cli<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.dummy_read(bus);
    cpu.set_flag(INTERRUPT_FLAG, false);
}

/// Name: Clear Overflow Flag (CLV)
/// Function: V = 0
/// Flags Updated: V
/// 
/// CLV clears the overflow flag to 0. The overflow flag is automatically
/// set by ADC and SBC when signed arithmetic overflow occurs, but can only
/// be cleared by CLV or by pulling a new status register from the stack
/// (PLP, RTI).
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, CLV performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn clv<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.dummy_read(bus);
    cpu.set_flag(OVERFLOW_FLAG, false);
}

/// Name: Compare Accumulator (CMP)
/// Function: A - M (result not stored)
/// Flags Updated: N, Z, C
/// 
/// CMP compares the accumulator with the operand by performing a subtraction
/// (A - M) but without storing the result. Only the flags are updated.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if A == M (result is 0)
/// - C (Carry): Set if A >= M (no borrow occurred)
/// 
/// The carry flag behavior is the inverse of what you might expect from
/// subtraction - it's set when no borrow is needed (A >= M), making it
/// useful for unsigned comparisons:
/// - C set: A >= M
/// - C clear: A < M
/// 
/// For signed comparisons, use the N and V flags together.
fn cmp<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);
    let res = cpu.a.wrapping_sub(cpu.operand);
    cpu.set_flag(NEGATIVE_FLAG, (res & 0x80) != 0);
    // cmp is implemented as sbc without storing so carry flag must be set if
    // there is no borrow, mirroring the sbc logic
    cpu.set_flag(ZERO_FLAG, res == 0);
    cpu.set_flag(CARRY_FLAG, cpu.a >= cpu.operand);
    
}

/// Name: Compare X Register (CPX)
/// Function: X - M (result not stored)
/// Flags Updated: N, Z, C
/// 
/// CPX compares the X register with the operand by performing a subtraction
/// (X - M) but without storing the result. Only the flags are updated.
/// 
/// This instruction works identically to CMP but uses the X register instead
/// of the accumulator. See CMP for detailed flag behavior.
fn cpx<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.fetch_operand(bus);
    let res = cpu.x.wrapping_sub(cpu.operand);
    cpu.set_flag(NEGATIVE_FLAG, (res & 0x80) != 0);
    // cmp is implemented as sbc without storing so carry flag must be set if
    // there is no borrow, mirroring the sbc logic
    cpu.set_flag(ZERO_FLAG, res == 0);
    cpu.set_flag(CARRY_FLAG, cpu.x >= cpu.operand);
}

/// Name: Compare Y Register (CPY)
/// Function: Y - M (result not stored)
/// Flags Updated: N, Z, C
/// 
/// CPY compares the Y register with the operand by performing a subtraction
/// (Y - M) but without storing the result. Only the flags are updated.
/// 
/// This instruction works identically to CMP but uses the Y register instead
/// of the accumulator. See CMP for detailed flag behavior.
fn cpy<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.fetch_operand(bus);
    let res = cpu.y.wrapping_sub(cpu.operand);
    cpu.set_flag(NEGATIVE_FLAG, (res & 0x80) != 0);
    // cmp is implemented as sbc without storing so carry flag must be set if
    // there is no borrow, mirroring the sbc logic
    cpu.set_flag(ZERO_FLAG, res == 0);
    cpu.set_flag(CARRY_FLAG, cpu.y >= cpu.operand);
}

/// Name: Decrement Memory (DEC)
/// Function: M = M - 1
/// Flags Updated: N, Z
/// 
/// DEC decrements the value at the specified memory location by 1.
/// This is a Read-Modify-Write (RMW) instruction.
/// 
/// The operation wraps around: 0x00 becomes 0xff.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// 
/// # RMW Instruction Timing
/// See ASL for details about RMW instruction timing and the dummy write cycle.
fn dec<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    rmw_read_write(cpu, bus);
    let res = cpu.operand.wrapping_sub(1);
    cpu.write_byte(cpu.addr_oper, res, bus);
    cpu.set_flag(NEGATIVE_FLAG, (res & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, res == 0);
}

/// Name: Decrement X Register (DEX)
/// Function: X = X - 1
/// Flags Updated: N, Z
/// 
/// DEX decrements the X register by 1. The operation wraps around:
/// 0x00 becomes 0xff.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, DEX performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn dex<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.dummy_read(bus);
    cpu.x = cpu.x.wrapping_sub(1);
    cpu.set_flag(NEGATIVE_FLAG, (cpu.x & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.x == 0);
}

/// Name: Decrement Y Register (DEY)
/// Function: Y = Y - 1
/// Flags Updated: N, Z
/// 
/// DEY decrements the Y register by 1. The operation wraps around:
/// 0x00 becomes 0xff.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, DEY performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn dey<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.dummy_read(bus);
    cpu.y =cpu.y.wrapping_sub(1);
    cpu.set_flag(NEGATIVE_FLAG, (cpu.y & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.y == 0);
}

/// Name: Exclusive OR (EOR)
/// Function: A = A ^ M
/// Flags Updated: N, Z
/// 
/// EOR performs a bitwise exclusive OR operation between the accumulator
/// and the operand from memory. Each bit of the result is 1 if the
/// corresponding bits in the operands are different.
/// 
/// The operation is:
/// - For each bit position i: result[i] = A[i] ^ M[i]
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// 
/// EOR is commonly used to toggle specific bits or to compare values
/// (EOR with itself always produces 0).
fn eor<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);
    cpu.a = cpu.a ^ cpu.operand;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.a & 0x80) != 0 );
    cpu.set_flag(ZERO_FLAG, cpu.a == 0);
}

/// Name: Increment Memory (INC)
/// Function: M = M + 1
/// Flags Updated: N, Z
/// 
/// INC increments the value at the specified memory location by 1.
/// This is a Read-Modify-Write (RMW) instruction.
/// 
/// The operation wraps around: 0xff becomes 0x00.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// 
/// # RMW Instruction Timing
/// See ASL for details about RMW instruction timing and the dummy write cycle.
fn inc<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    rmw_read_write(cpu, bus);
    let res = cpu.operand.wrapping_add(1);
    cpu.write_byte(cpu.addr_oper, res, bus);
    cpu.set_flag(NEGATIVE_FLAG, (res & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, res == 0);
}

/// Name: Increment X Register (INX)
/// Function: X = X + 1
/// Flags Updated: N, Z
/// 
/// INX increments the X register by 1. The operation wraps around:
/// 0xff becomes 0x00.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, INX performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn inx<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.x = cpu.x.wrapping_add(1);
    cpu.set_flag(NEGATIVE_FLAG, (cpu.x & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.x == 0);
    // drive bus
    cpu.dummy_read(bus);
}

/// Name: Increment Y Register (INY)
/// Function: Y = Y + 1
/// Flags Updated: N, Z
/// 
/// INY increments the Y register by 1. The operation wraps around:
/// 0xff becomes 0x00.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, INY performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn iny<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.y = cpu.y.wrapping_add(1);
    cpu.set_flag(NEGATIVE_FLAG, (cpu.y & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.y == 0);
    // drive bus
    cpu.dummy_read(bus);
}

/// Name: Jump (JMP)
/// Function: PC = address
/// Flags Updated: None
/// 
/// JMP transfers control to the specified address by loading the program
/// counter with the target address.
/// 
/// JMP supports two addressing modes:
/// - Absolute: JMP $1234 - jumps to address $1234
/// - Indirect: JMP ($1234) - jumps to address stored at $1234-$1235
/// 
/// # Indirect JMP Bug
/// The original 6502 has a bug with indirect JMP when the low byte of the
/// indirect address is $ff. Instead of reading the high byte from the next
/// page, it reads from the same page:
/// 
/// JMP ($10ff) reads the target address from $10ff and $1000 (correct)
/// JMP ($10ff) actually reads from $10ff and $1000 (should be $1100)
/// 
/// TODO: Implement bug and condition on the bus implementation.
fn jmp<B: Bus>(cpu: &mut CPU<B> , _bus: &mut B) {
    cpu.pc = cpu.addr_oper;
}

/// Name: Jump to Subroutine (JSR)
/// Function: Push PC+2 to stack, PC = address
/// Flags Updated: None
/// 
/// JSR pushes the return address (PC-1) onto the stack and then jumps to
/// the specified address. This is used to call subroutines.
/// 
/// The sequence of operations is:
/// 1. Fetch JSR opcode
/// 2. Fetch low byte of target address
/// 3. Perform a dummy read from the current stack address
/// 4. Push PCH to stack (PC now points to the high operand byte)
/// 5. Push PCL to stack
/// 6. Fetch the high byte of the target address and set PC
/// 
/// The return address pushed is PC-1, which points to the last byte of the
/// JSR instruction. RTS will increment this by 1 to return to the correct
/// location.
/// 
/// # Stack Behavior
/// JSR pushes the high byte first, then the low byte, so the stack layout
/// after JSR is:
/// - SP-1: PCL (low byte of return address)
/// - SP-2: PCH (high byte of return address)
fn jsr<B: Bus>(cpu: &mut CPU<B> , bus: &mut B) {
    cpu.dummy_stack_read(bus);
    cpu.push_word_to_stack(bus, cpu.pc);
    let addr_hi = cpu.fetch_from_and_increment_pc(bus) as Word;
    cpu.addr_oper = addr_hi << 8 | cpu.addr_oper;
    cpu.pc = cpu.addr_oper;
}

/// Name: Load Accumulator (LDA)
/// Function: A = M
/// Flags Updated: N, Z
/// 
/// LDA loads the accumulator with the value from memory.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the loaded value is 1
/// - Z (Zero): Set if the loaded value is 0x00
/// 
/// This is one of the most commonly used instructions for moving data
/// from memory into the accumulator for processing.
fn lda<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);
    cpu.a = cpu.operand;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.a & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.a == 0);
}

/// Name: Load X Register (LDX)
/// Function: X = M
/// Flags Updated: N, Z
/// 
/// LDX loads the X register with the value from memory.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the loaded value is 1
/// - Z (Zero): Set if the loaded value is 0x00
/// 
/// The X register is commonly used as an index register for addressing
/// modes and as a loop counter.
fn ldx<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);
    cpu.x = cpu.operand;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.x & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.x == 0);
}

/// Name: Load Y Register (LDY)
/// Function: Y = M
/// Flags Updated: N, Z
/// 
/// LDY loads the Y register with the value from memory.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the loaded value is 1
/// - Z (Zero): Set if the loaded value is 0x00
/// 
/// The Y register is commonly used as an index register for addressing
/// modes and as a loop counter.
fn ldy<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);
    cpu.y = cpu.operand;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.y & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.y == 0);
}

/// Name: Logical Shift Right (LSR)
/// Function: 0 -> [76543210] -> C
/// Flags Updated: N, Z, C
/// 
/// LSR shifts all bits of the operand one position to the right. Bit 7 is
/// filled with 0, and the original bit 0 is shifted into the carry flag.
/// 
/// This is a Read-Modify-Write (RMW) instruction that can operate on either:
/// - The accumulator (implied addressing)
/// - A memory location (various addressing modes)
/// 
/// The operation effectively divides the operand by 2 (unsigned).
/// 
/// Flag updates:
/// - N (Negative): Always cleared (bit 7 becomes 0)
/// - Z (Zero): Set if the result is 0x00
/// - C (Carry): Set to the value of the original bit 0
/// 
/// # RMW Instruction Timing
/// See ASL for details about RMW instruction timing and the dummy write cycle.
fn lsr<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // TODO: micro-optimization: if we were handling fetch here, cpu.operand
    // would be used as an internal register but we're not so we have to work
    // around that for now.
    let temp = if cpu.addr_mode == AddressingMode::Accumulator {
        cpu.dummy_read(bus);
        cpu.set_flag(CARRY_FLAG, (cpu.a & 0x01) != 0);
        let temp = cpu.a >> 1;
        cpu.a = temp;
        temp
    } else {
        rmw_read_write(cpu, bus);
        cpu.set_flag(CARRY_FLAG, (cpu.operand & 0x01) != 0);
        let temp = cpu.operand >> 1;
        cpu.write_byte(cpu.addr_oper, temp, bus);
        temp
    };
    cpu.set_flag(NEGATIVE_FLAG, false);
    cpu.set_flag(ZERO_FLAG, temp == 0);
}

/// Name: No Operation (NOP)
/// Function: No operation performed
/// Flags Updated: None
/// 
/// NOP performs no operation other than consuming 2 clock cycles.
/// It's commonly used for timing delays or as a placeholder instruction.
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, NOP performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn nop<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.dummy_read(bus);
}

/// Name: Logical OR with Accumulator(ORA)
/// Function: A = A | M
/// Flags Updated: N, Z
/// 
/// ORA performs a bitwise logical OR operation between the accumulator
/// and the operand from memory. Each bit of the result is 1 if either
/// corresponding bit in the operands is 1.
/// 
/// The operation is:
/// - For each bit position i: result[i] = A[i] | M[i]
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// 
/// ORA is commonly used to set specific bits in the accumulator.
fn ora<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);
    cpu.a = cpu.a | cpu.operand;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.a & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.a == 0);
}

/// Name: Push Accumulator (PHA)
/// Function: Push A to stack
/// Flags Updated: None
/// 
/// PHA pushes the accumulator onto the stack. The stack pointer is
/// decremented after the push.
/// 
/// # Stack Operation
/// The 6502 stack grows downward from $01ff. After PHA:
/// - Memory[0x0100 + SP] = A
/// - SP = SP - 1
/// 
/// # Bus Activity
/// All stack operations include a dummy read cycle to maintain consistent
/// timing with other instructions.
fn pha<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // wasted cycle in all stack ops
    cpu.dummy_read(bus);
    cpu.push_byte_to_stack(bus, cpu.a);
}

/// Name: Push Processor Status (PHP)
/// Function: Push status register to stack
/// Flags Updated: None
/// 
/// PHP pushes the processor status register onto the stack. The B flag
/// is set in the pushed value to distinguish software interrupts from
/// hardware interrupts.
/// 
/// The pushed status register has:
/// - B flag set (bit 4)
/// - Bit 5 always set (unused bit, always 1 when pushed)
/// - All other flags reflect current state
/// 
/// # Stack Operation
/// The 6502 stack grows downward from $01ff. After PHP:
/// - Memory[0x0100 + SP] = SR | B | 0x10
/// - SP = SP - 1
/// 
/// # Bus Activity
/// All stack operations include a dummy read cycle to maintain consistent
/// timing with other instructions.
fn php<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // wasted cycle in all stack ops
    cpu.dummy_read(bus);
    let status = cpu.sr | BREAK_FLAG | UNUSED_FLAG;
    cpu.push_byte_to_stack(bus, status);
}

/// Name: Pull Accumulator (PLA)
/// Function: Pull A from stack
/// Flags Updated: N, Z
/// 
/// PLA pulls a byte from the stack into the accumulator. The stack pointer
/// is incremented before the pull.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the pulled value is 1
/// - Z (Zero): Set if the pulled value is 0x00
/// 
/// # Stack Operation
/// The 6502 stack grows downward from $01ff. Before PLA:
/// - SP = SP + 1
/// - A = Memory[0x0100 + SP]
/// 
/// # Bus Activity
/// Stack operations include dummy read cycles for timing consistency.
fn pla<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // wasted cycle in all stack ops
    cpu.dummy_read(bus);
    cpu.dummy_stack_read(bus);
    cpu.a = cpu.pop_byte_from_stack(bus);
    cpu.set_flag(NEGATIVE_FLAG, (cpu.a & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.a == 0);
}

/// Name: Pull Processor Status (PLP)
/// Function: Pull status register from stack
/// Flags Updated: All flags restored from stack
/// 
/// PLP pulls a byte from the stack into the processor status register.
/// The B flag and bit 5 do not represent physical processor state. The B flag
/// is masked out and the emulator's bit 5 placeholder is kept set.
/// 
/// The B flag is only meaningful when pushed to the stack to distinguish
/// between BRK and IRQ. When pulled back, it's masked out because the
/// processor doesn't have a physical B flag register.
/// 
/// # Stack Operation
/// The 6502 stack grows downward from $01ff. Before PLP:
/// - SP = SP + 1
/// - SR = Memory[0x0100 + SP] & ~(B | 0x10)
fn plp<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // wasted cycle in all stack ops
    cpu.dummy_read(bus);
    cpu.dummy_stack_read(bus);
    let sr = (cpu.pop_byte_from_stack(bus) & !BREAK_FLAG) | UNUSED_FLAG;
    cpu.sr = sr;
}

/// Name: Rotate Left (ROL)
/// Function: C <- [76543210] <- C
/// Flags Updated: N, Z, C
/// 
/// ROL is a 9-bit rotation that includes the carry flag. All bits are
/// shifted left by one position, with the carry flag shifted into bit 0
/// and bit 7 shifted into the carry flag.
/// 
/// This is a Read-Modify-Write (RMW) instruction that can operate on either:
/// - The accumulator (implied addressing)
/// - A memory location (various addressing modes)
/// 
/// The bit layout during rotation:
/// ```pseudocode
/// [C] [b7 b6 b5 b4 b3 b2 b1 b0]
///  ^                           ^
///  |                           |
///  +---------------------------+
/// ```
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1
/// - Z (Zero): Set if the result is 0x00
/// - C (Carry): Set to the value of the original bit 7
/// 
/// # RMW Instruction Timing
/// See ASL for details about RMW instruction timing and the dummy write cycle.
fn rol<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // if we were handling fetch here, cpu.operand would be used as an internal
    // register but we're not so we have to work around that for now.
    let carry_in = cpu.get_flag(CARRY_FLAG) as u8;
    let temp = if cpu.addr_mode == AddressingMode::Accumulator {
        cpu.dummy_read(bus);
        cpu.set_flag(CARRY_FLAG, (cpu.a & 0x80) != 0);
        let temp = (cpu.a << 1) | carry_in;
        cpu.a = temp;
        temp
    } else {
        rmw_read_write(cpu, bus);
        cpu.set_flag(CARRY_FLAG, (cpu.operand & 0x80) != 0);
        let temp = (cpu.operand << 1) | carry_in;
        cpu.write_byte(cpu.addr_oper, temp, bus);
        temp
    };
    cpu.set_flag(NEGATIVE_FLAG, (temp & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, temp == 0);
}

/// Name: Rotate Right (ROR)
/// Function: C -> [76543210] -> C
/// Flags Updated: N, Z, C
/// 
/// ROR is a 9-bit rotation that includes the carry flag. All bits are
/// shifted right by one position, with the carry flag shifted into bit 7
/// and bit 0 shifted into the carry flag.
/// 
/// This is a Read-Modify-Write (RMW) instruction that can operate on either:
/// - The accumulator (implied addressing)
/// - A memory location (various addressing modes)
/// 
/// The bit layout during rotation:
/// ```pseudocode
/// [C] [b7 b6 b5 b4 b3 b2 b1 b0]
///  ^                           ^
///  |                           |
///  +---------------------------+
/// ```
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the result is 1 (same as carry in)
/// - Z (Zero): Set if the result is 0x00
/// - C (Carry): Set to the value of the original bit 0
/// 
/// # RMW Instruction Timing
/// See ASL for details about RMW instruction timing and the dummy write cycle.
fn ror<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // if we were handling fetch here, cpu.operand would be used as an internal
    // register but we're not so we have to work around that for now.
    let carry_in = (cpu.get_flag(CARRY_FLAG) as u8) << 7;
    let temp = if cpu.addr_mode == AddressingMode::Accumulator {
        cpu.dummy_read(bus);
        cpu.set_flag(CARRY_FLAG, (cpu.a & 0x01) != 0);
        let temp = (cpu.a >> 1) | carry_in;
        cpu.a = temp;
        temp
    } else {
        rmw_read_write(cpu, bus);
        cpu.set_flag(CARRY_FLAG, (cpu.operand & 0x01) != 0);
        let temp = (cpu.operand >> 1) | carry_in;
        cpu.write_byte(cpu.addr_oper, temp, bus);
        temp
    };
    cpu.set_flag(NEGATIVE_FLAG, (temp & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, temp == 0);
    
}

/// Name: Return from Interrupt (RTI)
/// Function: Pull status register and PC from stack
/// Flags Updated: All flags restored from stack
/// 
/// RTI returns from an interrupt by pulling the processor status register
/// and program counter from the stack. This is the counterpart to hardware
/// interrupts (IRQ, NMI) and the BRK instruction.
/// 
/// The sequence of operations is:
/// 1. Fetch the RTI opcode
/// 2. Dummy read from current PC
/// 3. Dummy read from the current stack address
/// 4. Pull status register from stack (with B masked out and bit 5 set)
/// 5. Pull PCL from stack
/// 6. Pull PCH from stack and continue execution at the restored PC
/// 
/// # Stack Layout
/// The stack contains:
/// - PCH (high byte of return address)
/// - PCL (low byte of return address)  
/// - Status register
/// 
/// # Flag Restoration
/// All physical flags are restored. B is masked out and the emulator's bit 5
/// placeholder is kept set.
fn rti<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // wasted cycle in all stack ops
    cpu.dummy_read(bus);
    cpu.dummy_stack_read(bus);
    cpu.sr = (cpu.pop_byte_from_stack(bus) & !BREAK_FLAG) | UNUSED_FLAG;
    cpu.refresh_irq_poll_i_flag();
    let pc_lo = cpu.pop_byte_from_stack(bus);
    let pc_hi = cpu.pop_byte_from_stack(bus);
    cpu.pc = (pc_hi as Word) << 8 | pc_lo as Word;
}

/// Name: Return from Subroutine (RTS)
/// Function: Pull PC from stack and increment
/// Flags Updated: None
/// 
/// RTS returns from a subroutine by pulling the program counter from the
/// stack and incrementing it by 1. This is the counterpart to the JSR
/// instruction.
/// 
/// The sequence of operations is:
/// 1. Dummy read from current PC
/// 2. Pull PCL from stack
/// 3. Pull PCH from stack
/// 4. Dummy read from current PC
/// 5. Increment PC (to point to instruction after JSR)
/// 6. Continue execution at PC
/// 
/// # Stack Layout
/// The stack must contain (from top to bottom):
/// - PCH (high byte of return address - 1)
/// - PCL (low byte of return address - 1)
/// 
/// JSR pushes PC-1, so RTS increments the pulled value to get the correct
/// return address.
fn rts<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    // wasted cycle in all stack ops
    cpu.dummy_read(bus);
    cpu.dummy_stack_read(bus);
    let pc_lo = cpu.pop_byte_from_stack(bus);
    let pc_hi = cpu.pop_byte_from_stack(bus);
    cpu.pc = (pc_hi as Word) << 8 | pc_lo as Word;
    cpu.fetch_from_and_increment_pc(bus);
}

/// Name: Subtract with Carry (SBC).
/// Function: A = A - M - !C.
/// Flags Updated: C, Z, V, N.
///  
/// SBC is a simple operation with all of the same complexity as ADC and then
/// some. First is the actual operation.
/// 
/// The function is:
/// A = A - M - !C           ...(1)
/// 
/// !C can be written as 1 - C so we can rewrite the equation as:
/// 
/// A = A - M - (1 - C)      ...(2)
/// A = A - M - 1 + C        ...(3)
/// A = A + C + (-M - 1)     ...(4)
/// 
/// Negating a number requires computing its two's complement.
/// 
/// Computing the two's complement requires:
/// 1. Computing the one's complement (inverting all the bits),
/// 2. Adding one.
/// 
/// Algebraically, two's complement of A is !A + 1.
/// So, equation 4 can be written as:
/// 
/// A = A + C + (!M + 1 - 1) ...(5)
/// 
/// which simplifies to:
/// A = A + C + !M           ...(6)
/// 
/// which happens to be the exact same operation as ADC with the memory operand
/// not-ed: A = A + !M + C.
/// 
/// This greatly simplifies the logic for SBC.
/// 
/// The other quirk (that is not really an issue, just strange) is that the
/// 6502 encodes a borrow by setting `CARRY` to 0, and no borrow by setting it
/// to 1. This is counterintuitive but allows the optimization discussed above
/// and by extension, reusing the ADC circuitry.
/// 
/// # Decimal Mode
/// Decimal mode for SBC is similar to decimal mode for ADC; the difference is
/// the subtraction of the correction factor instead of addition. This is a
/// quirk of BCD and its borrow rules.
/// 
/// On the NMOS 6502, N, V, and Z come from the binary subtraction even when
/// decimal mode is enabled. The accumulator is corrected one nibble at a time,
/// and C records whether the high-nibble subtraction completed without a
/// borrow.
fn sbc<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    extra_cycle!(cpu, bus);
    cpu.fetch_operand(bus);

    let old_a = cpu.a;
    let operand = cpu.operand;
    let borrow = !cpu.get_flag(CARRY_FLAG);
    let binary_result = old_a
        .wrapping_sub(operand)
        .wrapping_sub(borrow as u8);
    let negative = binary_result & 0x80 != 0;
    let overflow = ((old_a >= 0x80) ^ negative)
        && ((operand < 0x80) ^ negative);

    if cpu.get_flag(DECIMAL_FLAG) {
        let mut low_nibble = (old_a & 0x0f)
            .wrapping_sub(operand & 0x0f)
            .wrapping_sub(borrow as u8);
        let low_borrow = low_nibble >= 0x80;

        if low_borrow {
            low_nibble = low_nibble.wrapping_add(10) & 0x0f;
        }

        let mut high_nibble = (old_a >> 4)
            .wrapping_sub(operand >> 4)
            .wrapping_sub(low_borrow as u8);
        let high_borrow = high_nibble >= 0x80;

        if high_borrow {
            high_nibble = high_nibble.wrapping_add(10) & 0x0f;
        }

        cpu.a = (high_nibble << 4) | low_nibble;
        cpu.set_flag(CARRY_FLAG, !high_borrow);
    } else {
        cpu.a = binary_result;
        cpu.set_flag(
            CARRY_FLAG,
            old_a as Word >= operand as Word + borrow as Word,
        );
    }
    cpu.set_flag(NEGATIVE_FLAG, negative);
    cpu.set_flag(ZERO_FLAG, binary_result == 0);
    cpu.set_flag(OVERFLOW_FLAG, overflow);
}

/// Name: Set Carry Flag (SEC)
/// Function: C = 1
/// Flags Updated: C
/// 
/// SEC sets the carry flag to 1. This is commonly used before subtraction
/// operations (since SBC subtracts the complement of carry) or to set the
/// carry flag for use as a general-purpose flag.
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, SEC performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn sec<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.set_flag(CARRY_FLAG, true);
    cpu.dummy_read(bus);
}

/// Name: Set Decimal Flag (SED)
/// Function: D = 1
/// Flags Updated: D
/// 
/// SED sets the decimal flag to 1, switching the processor to Binary
/// Coded Decimal (BCD) mode.
/// 
/// In BCD mode, ADC and SBC treat each nibble (4 bits) as a decimal digit
/// (0-9). This allows decimal arithmetic without binary-to-decimal conversion.
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, SED performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn sed<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.set_flag(DECIMAL_FLAG, true);
    cpu.dummy_read(bus);
}

/// Name: Set Interrupt Disable Flag (SEI)
/// Function: I = 1
/// Flags Updated: I
/// 
/// SEI sets the interrupt disable flag to 1, disabling maskable interrupts
/// (IRQ).
/// 
/// When the I flag is set, the processor will ignore IRQ signals. NMI
/// (Non-Maskable Interrupt) signals are still processed regardless of the
/// I flag state.
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, SEI performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn sei<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.set_flag(INTERRUPT_FLAG, true);
    cpu.dummy_read(bus);
}

/// Name: Store Accumulator (STA)
/// Function: M = A
/// Flags Updated: None
/// 
/// STA stores the accumulator value to the specified memory location.
/// No flags are affected by store operations.
/// 
/// This is one of the most commonly used instructions for moving data
/// from the accumulator to memory.
fn sta<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    if matches!(
        cpu.addr_mode,
        AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::IndirectIndexed
    ) {
        cpu.fetch_byte(cpu.addr_oper_prov, bus);
    }
    cpu.write_byte(cpu.addr_oper, cpu.a, bus);
}

/// Name: Store X Register (STX)
/// Function: M = X
/// Flags Updated: None
/// 
/// STX stores the X register value to the specified memory location.
/// No flags are affected by store operations.
fn stx<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.write_byte(cpu.addr_oper, cpu.x, bus);
}

/// Name: Store Y Register (STY)
/// Function: M = Y
/// Flags Updated: None
/// 
/// STY stores the Y register value to the specified memory location.
/// No flags are affected by store operations.
fn sty<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.write_byte(cpu.addr_oper, cpu.y, bus);
}

/// Name: Transfer Accumulator to X (TAX)
/// Function: X = A
/// Flags Updated: N, Z
/// 
/// TAX transfers the accumulator value to the X register.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the transferred value is 1
/// - Z (Zero): Set if the transferred value is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, TAX performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn tax<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.x = cpu.a;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.x & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.x == 0);
    cpu.dummy_read(bus);
}

/// Name: Transfer Accumulator to Y (TAY)
/// Function: Y = A
/// Flags Updated: N, Z
/// 
/// TAY transfers the accumulator value to the Y register.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the transferred value is 1
/// - Z (Zero): Set if the transferred value is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, TAY performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn tay<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.y = cpu.a;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.y & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.y == 0);
    cpu.dummy_read(bus);
}

/// Name: Transfer Stack Pointer to X (TSX)
/// Function: X = SP
/// Flags Updated: N, Z
/// 
/// TSX transfers the stack pointer value to the X register. This allows
/// the program to examine or manipulate the stack pointer indirectly.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the stack pointer is 1
/// - Z (Zero): Set if the stack pointer is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, TSX performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn tsx<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.x = cpu.sp;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.x & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.x == 0);
    cpu.dummy_read(bus);
}

/// Name: Transfer X to Accumulator (TXA)
/// Function: A = X
/// Flags Updated: N, Z
/// 
/// TXA transfers the X register value to the accumulator.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the transferred value is 1
/// - Z (Zero): Set if the transferred value is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, TXA performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn txa<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.a = cpu.x;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.a & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.a == 0);
    cpu.dummy_read(bus);
}

/// Name: Transfer X to Stack Pointer (TXS)
/// Function: SP = X
/// Flags Updated: None
/// 
/// TXS transfers the X register value to the stack pointer. This allows
/// the program to set up or modify the stack pointer.
/// 
/// Note: Unlike other transfer instructions, TXS does not update any flags.
/// This is a unique characteristic of the TXS instruction.
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, TXS performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn txs<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.sp = cpu.x;
    // Note: TXS does not set flags, unlike other transfer instructions
    cpu.dummy_read(bus);
}

/// Name: Transfer Y to Accumulator (TYA)
/// Function: A = Y
/// Flags Updated: N, Z
/// 
/// TYA transfers the Y register value to the accumulator.
/// 
/// Flag updates:
/// - N (Negative): Set if bit 7 of the transferred value is 1
/// - Z (Zero): Set if the transferred value is 0x00
/// 
/// # Bus Activity
/// Like all implied addressing mode instructions, TYA performs a dummy
/// read of the current PC to keep the bus active during the instruction.
fn tya<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    cpu.a = cpu.y;
    cpu.set_flag(NEGATIVE_FLAG, (cpu.a & 0x80) != 0);
    cpu.set_flag(ZERO_FLAG, cpu.a == 0);
    cpu.dummy_read(bus);
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bus::setup_cpu_and_memory;

    #[test]
    fn test_adc_no_carry_no_overflow() {
        // 0x50 + 0x10 = 0x60 (positive + positive = positive, no overflow)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x10);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, false);
        
        adc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x60);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(OVERFLOW_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_adc_with_carry_out() {
        // 0xff + 0x01 = 0x00 with carry
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x01);
        cpu.a = 0xff;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, false);
        
        adc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(OVERFLOW_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_adc_with_overflow() {
        // 0x50 + 0x50 = 0xa0 (positive + positive = negative, overflow)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x50);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, false);
        
        adc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0xa0);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(OVERFLOW_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_adc_with_carry_in() {
        // 0x50 + 0x50 + 1 = 0xa1 (with carry in)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x50);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, true);
        
        adc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0xa1);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(OVERFLOW_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_adc_negative_overflow() {
        // 0x80 + 0x80 = 0x00 (negative + negative = positive, overflow with carry)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x80);
        cpu.a = 0x80;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, false);
        
        adc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(OVERFLOW_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_adc_decimal() {
        // BCD: 0x19 + 0x11 + 1 = 0x31 (19 + 11 + 1 = 31 in decimal)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x11);
        cpu.a = 0x19;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(DECIMAL_FLAG | CARRY_FLAG, true);
        
        adc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x31);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_adc_decimal_with_carry_out() {
        // BCD: 0x99 + 0x01 = 0x00 with carry (99 + 1 = 100, wraps to 00)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x01);
        cpu.a = 0x99;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(DECIMAL_FLAG, true);
        cpu.set_flag(CARRY_FLAG, false);
        
        adc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(OVERFLOW_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_adc_decimal_flags_use_intermediate_result() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x00);
        cpu.a = 0x79;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(DECIMAL_FLAG | CARRY_FLAG, true);

        adc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(OVERFLOW_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(CARRY_FLAG));
    }

    #[test]
    fn test_and() {
        // 0xf0 & 0x0f = 0x00
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x0f);
        cpu.a = 0xf0;
        cpu.addr_oper = 0x8000;

        and(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_and_negative() {
        // 0xff & 0x80 = 0x80 (negative result)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x80);
        cpu.a = 0xff;
        cpu.addr_oper = 0x8000;

        and(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_asl() {
        // 0x42 << 1 = 0x84 (no carry out)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x42);
        cpu.addr_oper = 0x8000;
        
        asl(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x84);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_asl_with_carry() {
        // 0x81 << 1 = 0x02 with carry
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x81);
        cpu.addr_oper = 0x8000;
        
        asl(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x02);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_asl_zero() {
        // 0x80 << 1 = 0x00 with carry
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x80);
        cpu.addr_oper = 0x8000;
        
        asl(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x00);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    macro_rules! test_branch_taken {
        ($flag: ident, $val: expr, $func: expr) => {
            const OFFSET : i8 = -128;
            const ADDR_OPER   : u16 = 0x8000;
            const EXPECTED_PC : u16 = 0x7f80;

            let (mut cpu, mut mem) = setup_cpu_and_memory();
            
            mem.write( ADDR_OPER, OFFSET as u8);
            cpu.pc = ADDR_OPER;
            cpu.addr_oper = ADDR_OPER;
            cpu.set_flag($flag, $val);

            $func(&mut cpu, &mut mem);

            assert!(cpu.pc == EXPECTED_PC);
            assert!(cpu.cycles == 3);  
        };
    }

    macro_rules! test_branch_not_taken {
        ($flag: ident, $val: expr, $func: expr) => {
            const OFFSET : i8 = -128;
            const ADDR_OPER   : u16 = 0x8000;
            const EXPECTED_PC : u16 = ADDR_OPER;

            let (mut cpu, mut mem) = setup_cpu_and_memory();
            
            mem.write( ADDR_OPER, OFFSET as u8);
            cpu.pc = ADDR_OPER;
            cpu.addr_oper = ADDR_OPER;
            cpu.set_flag($flag, $val);

            $func(&mut cpu, &mut mem);

            assert!(cpu.pc == EXPECTED_PC);
            assert!(cpu.cycles == 1);  
        };
    }

    #[test]
    fn test_bcc_taken() {
        test_branch_taken!(CARRY_FLAG, false, bcc);
    }

    #[test]
    fn test_bcc_not_taken() {
        test_branch_not_taken!(CARRY_FLAG, true, bcc);
    }

    #[test]
    fn test_bcs_taken() {
        test_branch_taken!(CARRY_FLAG, true, bcs);
    }

    #[test]
    fn test_bcs_not_taken() {
        test_branch_not_taken!(CARRY_FLAG, false, bcs);
    }

    #[test]
    fn test_beq_taken() {
        test_branch_taken!(ZERO_FLAG, true, beq);
    }

    #[test]
    fn test_beq_not_taken() {
        test_branch_not_taken!(ZERO_FLAG, false, beq);
    }

    #[test]
    fn test_bit() {
        const ACCUMULATOR_VAL: u8 = 0b1010_1010;
        const OPERAND        : u8 = 0b0010_1010;

        const ADDR_OPER: u16 = 0x8000;

        let (mut cpu, mut mem) = setup_cpu_and_memory();
            
        mem.write( ADDR_OPER, OPERAND as u8);
        cpu.a = ACCUMULATOR_VAL;
        cpu.pc = ADDR_OPER;
        cpu.addr_oper = ADDR_OPER;

        bit(&mut cpu, &mut mem);

        assert!(cpu.get_flag(NEGATIVE_FLAG) == false);
        assert!(cpu.get_flag(OVERFLOW_FLAG) == false);
        assert!(cpu.get_flag(ZERO_FLAG)     == false);
    }

    #[test]
    fn test_bmi_taken() {
        test_branch_taken!(NEGATIVE_FLAG, true, bmi);
    }
    
    #[test]
    fn test_bmi_not_taken() {
        test_branch_not_taken!(NEGATIVE_FLAG, false, bmi);
    }
    
    #[test]
    fn test_bne_taken() {
        test_branch_taken!(ZERO_FLAG, false, bne);
    }
    
    #[test]
    fn test_bne_not_taken() {
        test_branch_not_taken!(ZERO_FLAG, true, bne);
    }

    #[test]
    fn test_bpl_taken() {
        test_branch_taken!(NEGATIVE_FLAG, false, bpl);
    }
    
    #[test]
    fn test_bpl_not_taken() {
        test_branch_not_taken!(NEGATIVE_FLAG, true, bpl);
    }

    #[test]
    fn test_brk() {
        // Never use any 00 bytes in tests since memory itself is zeroed.
        const INITIAL_PC           : u16 = 0x80_01;
        // should be pc + 2, but we skip operand fetch here so pc + 1.
        // const RETURN_ADDRESS       : u16 = 0x80_02;
        const INTERRUPT_VEC_ADDR   : u16 = 0x69_96;
        const INITIAL_FLAG_PATTERN : u8  = 0b0110_0011;
        const EXPECTED_FLAG_PATTERN: u8  = 0b0111_0011;

        let (mut cpu, mut mem) = setup_cpu_and_memory();
        // write higher and lower byte of interrupt vector.
        mem.write(0xff_fe, 0x96);
        mem.write(0xff_ff, 0x69);
        cpu.pc = INITIAL_PC;
        cpu.sr = INITIAL_FLAG_PATTERN;

        brk(&mut cpu, &mut mem); 

        let sp = cpu.full_stack_pointer_address();

        // assert that the correct return address was pushed to the stack.
        assert!(mem.read(sp + 3) == 0x80);
        assert!(mem.read(sp + 2) == 0x02);

        // assert that the SR was correctly pushed to the stack and the break
        // flag was correctly set.
        assert!((mem.read(sp + 1) == EXPECTED_FLAG_PATTERN));
        
        assert!(cpu.get_flag(INTERRUPT_FLAG));
        assert!(cpu.pc == INTERRUPT_VEC_ADDR);
        assert!(cpu.cycles == 6);
    }

    #[test]
    fn test_bvc_taken() {
        test_branch_taken!(OVERFLOW_FLAG, false, bvc);
    }
    
    #[test]
    fn test_bvc_not_taken() {
        test_branch_not_taken!(OVERFLOW_FLAG, true, bvc);
    }

    #[test]
    fn test_bvs_taken() {
        test_branch_taken!(OVERFLOW_FLAG, true, bvs);
    }
    
    #[test]
    fn test_bv_not_taken() {
        test_branch_not_taken!(OVERFLOW_FLAG, false, bvs);
    }

    macro_rules! test_clear_flag {
        ($flag: ident, $flag_status: expr, $func: expr) => {{
            let (mut cpu, mut mem) = setup_cpu_and_memory();
            cpu.set_flag($flag, $flag_status);
            $func(&mut cpu, &mut mem);
            assert!(cpu.get_flag($flag) == false);
        }};
    }

    #[test]
    fn test_clc_set() {
        test_clear_flag!(CARRY_FLAG, true, clc)
    }

    #[test]
    fn test_clc_clr() {
        test_clear_flag!(CARRY_FLAG, false, clc)
    }

    #[test]
    fn test_cld_set() {
        test_clear_flag!(DECIMAL_FLAG, true, cld)
    }

    #[test]
    fn test_cld_clr() {
        test_clear_flag!(DECIMAL_FLAG, false, cld)
    }

    #[test]
    fn test_cli_set() {
        test_clear_flag!(INTERRUPT_FLAG, true, cli)
    }

    #[test]
    fn test_cli_clr() {
        test_clear_flag!(INTERRUPT_FLAG, false, cli)
    }

    #[test]
    fn test_clv_set() {
        test_clear_flag!(OVERFLOW_FLAG, true, clv)
    }

    #[test]
    fn test_clv_clr() {
        test_clear_flag!(OVERFLOW_FLAG, false, clv)
    }

    #[test]
    fn test_cmp_equal() {
        // 0x42 - 0x42 = 0x00 (Z=1, C=1, N=0)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x42);
        cpu.a = 0x42;
        cpu.addr_oper = 0x8000;

        cmp(&mut cpu, &mut mem);

        assert!(cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_cmp_greater() {
        // 0x50 - 0x30 = 0x20 (Z=0, C=1, N=0)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x30);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;

        cmp(&mut cpu, &mut mem);

        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_cmp_less() {
        // 0x30 - 0x50 = 0xe0 (Z=0, C=0, N=1)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x50);
        cpu.a = 0x30;
        cpu.addr_oper = 0x8000;

        cmp(&mut cpu, &mut mem);

        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_cpx_equal() {
        // 0x42 - 0x42 = 0x00 (Z=1, C=1, N=0)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x42);
        cpu.x = 0x42;
        cpu.addr_oper = 0x8000;

        cpx(&mut cpu, &mut mem);

        assert!(cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_cpx_greater() {
        // 0x50 - 0x30 = 0x20 (Z=0, C=1, N=0)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x30);
        cpu.x = 0x50;
        cpu.addr_oper = 0x8000;

        cpx(&mut cpu, &mut mem);

        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_cpx_less() {
        // 0x30 - 0x50 = 0xe0 (Z=0, C=0, N=1)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x50);
        cpu.x = 0x30;
        cpu.addr_oper = 0x8000;

        cpx(&mut cpu, &mut mem);

        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_cpy_equal() {
        // 0x42 - 0x42 = 0x00 (Z=1, C=1, N=0)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x42);
        cpu.y = 0x42;
        cpu.addr_oper = 0x8000;

        cpy(&mut cpu, &mut mem);

        assert!(cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_cpy_greater() {
        // 0x50 - 0x30 = 0x20 (Z=0, C=1, N=0)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x30);
        cpu.y = 0x50;
        cpu.addr_oper = 0x8000;

        cpy(&mut cpu, &mut mem);

        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_cpy_less() {
        // 0x30 - 0x50 = 0xe0 (Z=0, C=0, N=1)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x50);
        cpu.y = 0x30;
        cpu.addr_oper = 0x8000;

        cpy(&mut cpu, &mut mem);

        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_dec() {
        // 0x42 - 1 = 0x41
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8765, 0x42);
        cpu.addr_oper = 0x8765;

        dec(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8765), 0x41);
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_dec_zero() {
        // 0x01 - 1 = 0x00
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8765, 0x01);
        cpu.addr_oper = 0x8765;

        dec(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8765), 0x00);
        assert!(cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_dec_wrap() {
        // 0x00 - 1 = 0xff (wraps around)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8765, 0x00);
        cpu.addr_oper = 0x8765;

        dec(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8765), 0xff);
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_dex() {
        // 0x42 - 1 = 0x41
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x42;

        dex(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x41);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_dex_zero() {
        // 0x01 - 1 = 0x00
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x01;

        dex(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_dex_wrap() {
        // 0x00 - 1 = 0xff
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x00;

        dex(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0xff);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_dey() {
        // 0x42 - 1 = 0x41
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0x42;

        dey(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0x41);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_dey_zero() {
        // 0x01 - 1 = 0x00
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0x01;

        dey(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_dey_wrap() {
        // 0x00 - 1 = 0xff
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0x00;

        dey(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0xff);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_eor() {
        // 0xff ^ 0x0f = 0xf0
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8765, 0x0f);
        cpu.addr_oper = 0x8765;
        cpu.a = 0xff;

        eor(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0xf0);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_eor_zero() {
        // 0x42 ^ 0x42 = 0x00
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8765, 0x42);
        cpu.addr_oper = 0x8765;
        cpu.a = 0x42;

        eor(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_inc() {
        // 0x41 + 1 = 0x42
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8765, 0x41);
        cpu.addr_oper = 0x8765;

        inc(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8765), 0x42);
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_inc_wrap() {
        // 0xff + 1 = 0x00 (wraps around)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8765, 0xff);
        cpu.addr_oper = 0x8765;

        inc(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8765), 0x00);
        assert!(cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_inc_negative() {
        // 0x7f + 1 = 0x80 (becomes negative)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8765, 0x7f);
        cpu.addr_oper = 0x8765;

        inc(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8765), 0x80);
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
    }

    #[test]
    fn test_inx() {
        // 0x41 + 1 = 0x42
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x41;

        inx(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x42);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_inx_wrap() {
        // 0xff + 1 = 0x00
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0xff;

        inx(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_inx_negative() {
        // 0x7f + 1 = 0x80
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x7f;

        inx(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_iny() {
        // 0x41 + 1 = 0x42
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0x41;

        iny(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0x42);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_iny_wrap() {
        // 0xff + 1 = 0x00
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0xff;

        iny(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_iny_negative() {
        // 0x7f + 1 = 0x80
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0x7f;

        iny(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_jmp() {
        const FINAL_PC: u16 = 0x6969;

        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.addr_oper = FINAL_PC;

        jmp(&mut cpu, &mut mem);

        assert!(cpu.pc == FINAL_PC);
    }

    #[test]
    fn test_jsr() { 
        const PC             : u16 = 0x6996;
        const SUBROUTINE_ADDR: u16 = 0x4224;

        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.pc = PC;
        cpu.addr_oper = SUBROUTINE_ADDR & 0x00ff;
        mem.write(PC, (SUBROUTINE_ADDR >> 8) as u8);

        jsr(&mut cpu, &mut mem);
        let sp = cpu.full_stack_pointer_address();

        assert!(cpu.pc == SUBROUTINE_ADDR);
        assert!(mem.read(sp + 2) == (PC >> 8) as u8);
        assert!(mem.read(sp + 1) == (PC & 0xff) as u8);
    }

    #[test]
    fn test_lda() {
        const ADDR_OPER : u16 = 0x9123;
        const MEMORY_VAL:  u8 = 0x69;
        
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(ADDR_OPER, MEMORY_VAL);
        cpu.addr_oper = ADDR_OPER;

        lda(&mut cpu, &mut mem);

        assert!(cpu.a == MEMORY_VAL);
        assert!(cpu.get_flag(NEGATIVE_FLAG) == ((MEMORY_VAL & 0x80) != 0));
        assert!(cpu.get_flag(ZERO_FLAG) == (MEMORY_VAL == 0));
    }

    #[test]
    fn test_ldx() {
        const ADDR_OPER : u16 = 0x9123;
        const MEMORY_VAL:  u8 = 0x69;
        
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(ADDR_OPER, MEMORY_VAL);
        cpu.addr_oper = ADDR_OPER;

        ldx(&mut cpu, &mut mem);

        assert!(cpu.x == MEMORY_VAL);
        assert!(cpu.get_flag(NEGATIVE_FLAG) == ((MEMORY_VAL & 0x80) != 0));
        assert!(cpu.get_flag(ZERO_FLAG) == (MEMORY_VAL == 0));
    }

    #[test]
    fn test_ldy() {
        const ADDR_OPER : u16 = 0x9123;
        const MEMORY_VAL:  u8 = 0x69;
        
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(ADDR_OPER, MEMORY_VAL);
        cpu.addr_oper = ADDR_OPER;

        ldy(&mut cpu, &mut mem);

        assert!(cpu.y == MEMORY_VAL);
        assert!(cpu.get_flag(NEGATIVE_FLAG) == ((MEMORY_VAL & 0x80) != 0));
        assert!(cpu.get_flag(ZERO_FLAG) == (MEMORY_VAL == 0));
    }

    #[test]
    fn test_lsr() {
        // 0x42 >> 1 = 0x21 (no carry)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x42);
        cpu.addr_oper = 0x8000;
        
        lsr(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x21);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_lsr_with_carry() {
        // 0x43 >> 1 = 0x21 with carry
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x43);
        cpu.addr_oper = 0x8000;
        
        lsr(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x21);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_lsr_zero() {
        // 0x01 >> 1 = 0x00 with carry
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x01);
        cpu.addr_oper = 0x8000;
        
        lsr(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x00);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_nop() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        nop(&mut cpu, &mut mem);

        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_ora() {
        // 0x0f | 0xf0 = 0xff
        let (mut cpu, mut mem) =  setup_cpu_and_memory();
        mem.write(0x1234, 0xf0);
        cpu.a = 0x0f;
        cpu.addr_oper = 0x1234;

        ora(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0xff);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_ora_zero() {
        // 0x00 | 0x00 = 0x00
        let (mut cpu, mut mem) =  setup_cpu_and_memory();
        mem.write(0x1234, 0x00);
        cpu.a = 0x00;
        cpu.addr_oper = 0x1234;

        ora(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_pha() {
        const ACCUMULATOR_VAL: u8 = 0x69;
        
        let (mut cpu, mut mem) =  setup_cpu_and_memory();
        cpu.a = ACCUMULATOR_VAL;

        pha(&mut cpu, &mut mem);
        let sp = cpu.full_stack_pointer_address();
        assert!(mem.read(sp + 1) == ACCUMULATOR_VAL);
        assert!(cpu.cycles == 2);
    }

    #[test]
    fn test_php() {
        const STATUS_REGISTER: u8 = 0b1100_1001;
        
        let (mut cpu, mut mem) =  setup_cpu_and_memory();
        cpu.sr = STATUS_REGISTER;

        php(&mut cpu, &mut mem);
        let sp = cpu.full_stack_pointer_address();
        assert!(mem.read(sp + 1) == (STATUS_REGISTER | BREAK_FLAG | UNUSED_FLAG));
        assert!(cpu.cycles == 2);
    }

    #[test]
    fn test_pla() {
        const SP          : u16 = 0x0104;
        const EXPECTED_VAL:  u8 = 0x24;

        let (mut cpu, mut mem) =  setup_cpu_and_memory();

        mem.write(SP + 1, EXPECTED_VAL);
        cpu.sp = (SP & 0xff) as u8;

        pla(&mut cpu, &mut mem);

        assert!(cpu.a == EXPECTED_VAL);
        assert!(cpu.get_flag(NEGATIVE_FLAG) == ((EXPECTED_VAL & 0x80) != 0));
        assert!(cpu.get_flag(ZERO_FLAG) == (EXPECTED_VAL == 0));
    }

    #[test]
    fn test_plp() {
        const SP          : u16 = 0x0104;
        const EXPECTED_VAL:  u8 = 0b0011_1001;

        let (mut cpu, mut mem) =  setup_cpu_and_memory();

        mem.write(SP + 1, EXPECTED_VAL);
        cpu.sp = (SP & 0xff) as u8;

        plp(&mut cpu, &mut mem);

        assert!(cpu.sr == (EXPECTED_VAL & !BREAK_FLAG) | UNUSED_FLAG);
        assert!(cpu.get_flag(NEGATIVE_FLAG) == ((EXPECTED_VAL & 0x80) != 0));
        assert!(cpu.get_flag(ZERO_FLAG) == (EXPECTED_VAL == 0));
    }

    #[test]
    fn test_rol() {
        // ROL 0x42 with carry=1: 0x42 << 1 | 1 = 0x85
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x42);
        cpu.set_flag(CARRY_FLAG, true);
        cpu.addr_oper = 0x8000;
        
        rol(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x85);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_rol_with_carry_out() {
        // ROL 0x81 with carry=0: 0x81 << 1 | 0 = 0x02, carry out = 1
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x81);
        cpu.set_flag(CARRY_FLAG, false);
        cpu.addr_oper = 0x8000;
        
        rol(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x02);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_rol_zero() {
        // ROL 0x80 with carry=0: 0x80 << 1 | 0 = 0x00, carry out = 1
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x80);
        cpu.set_flag(CARRY_FLAG, false);
        cpu.addr_oper = 0x8000;
        
        rol(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x00);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_ror() {
        // ROR 0x42 with carry=1: (0x42 >> 1) | 0x80 = 0xa1
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x42);
        cpu.set_flag(CARRY_FLAG, true);
        cpu.addr_oper = 0x8000;
        
        ror(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0xa1);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_ror_with_carry_out() {
        // ROR 0x43 with carry=0: (0x43 >> 1) | 0x00 = 0x21, carry out = 1
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x43);
        cpu.set_flag(CARRY_FLAG, false);
        cpu.addr_oper = 0x8000;
        
        ror(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x21);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_ror_zero() {
        // ROR 0x01 with carry=0: (0x01 >> 1) | 0x00 = 0x00, carry out = 1
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x01);
        cpu.set_flag(CARRY_FLAG, false);
        cpu.addr_oper = 0x8000;
        
        ror(&mut cpu, &mut mem);

        assert_eq!(mem.read(0x8000), 0x00);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_rti() {
        const SP            : u16 = 0x0104;
        const RETURN_ADDRESS: u16 = 0x4567;
        const SR            :  u8 = 0b1011_1100;
        const EXPECTED_SR   :  u8 = 0b1010_1100;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(SP + 1, SR);
        mem.write(SP + 2, 0x67);
        mem.write(SP + 3, 0x45);

        cpu.sr = SR;
        cpu.sp = (SP & 0xff) as u8;

        rti(&mut cpu, &mut mem);

        assert_eq!(cpu.sr, EXPECTED_SR);
        assert_eq!(cpu.pc, RETURN_ADDRESS);
        assert_eq!(cpu.cycles, 5);
    }

     #[test]
    fn test_rts() {
        const SP            : u16 = 0x0104;
        const RETURN_ADDRESS: u16 = 0x4567;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(SP + 1, (RETURN_ADDRESS & 0xff) as u8);
        mem.write(SP + 2, (RETURN_ADDRESS >> 8) as u8);

        cpu.sp = (SP & 0xff) as u8;

        rts(&mut cpu, &mut mem);

        assert!(cpu.pc == RETURN_ADDRESS + 1);
        assert!(cpu.cycles == 5);
    }

    #[test]
    fn test_sbc_no_borrow() {
        // 0x50 - 0x30 - 0 = 0x20 (no borrow, no overflow)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x30);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, true);

        sbc(&mut cpu, &mut mem);
        
        assert_eq!(cpu.a, 0x20);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(OVERFLOW_FLAG));
    }

    #[test]
    fn test_sbc_with_borrow() {
        // 0x50 - 0x70 - 0 = 0xe0 (borrow, no overflow)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x70);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, true);

        sbc(&mut cpu, &mut mem);
        
        assert_eq!(cpu.a, 0xe0);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(OVERFLOW_FLAG));
    }

    #[test]
    fn test_sbc_with_overflow() {
        // 0x50 - 0xb0 - 0 = 0xa0 (positive - negative = negative, overflow)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0xb0);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, true);

        sbc(&mut cpu, &mut mem);
        
        assert_eq!(cpu.a, 0xa0);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(cpu.get_flag(OVERFLOW_FLAG));
    }

    #[test]
    fn test_sbc_zero() {
        // 0x50 - 0x50 - 0 = 0x00
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x50);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(CARRY_FLAG, true);

        sbc(&mut cpu, &mut mem);
        
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(OVERFLOW_FLAG));
    }

    #[test]
    fn test_sbc_decimal() {
        // BCD: 0x50 - 0x27 - 0 = 0x23 (50 - 27 = 23 in decimal)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x27);
        cpu.a = 0x50;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(DECIMAL_FLAG | CARRY_FLAG, true);
        
        sbc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x23);
        assert!(cpu.get_flag(CARRY_FLAG));
    }

    #[test]
    fn test_sbc_decimal_with_borrow() {
        // BCD: 0x32 - 0x45 - 0 = 0x87 (32 - 45 = -13, represented as 87 in BCD)
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x45);
        cpu.a = 0x32;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(DECIMAL_FLAG | CARRY_FLAG, true);
        
        sbc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x87);
        assert!(!cpu.get_flag(CARRY_FLAG));
    }

    #[test]
    fn test_sbc_decimal_with_invalid_bcd_operand() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x0b);
        cpu.a = 0x00;
        cpu.addr_oper = 0x8000;
        cpu.set_flag(DECIMAL_FLAG | CARRY_FLAG, true);

        sbc(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x9f);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(OVERFLOW_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(CARRY_FLAG));
    }

    #[test]
    fn test_sec_set() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.set_flag(CARRY_FLAG, true);

        sec(&mut cpu, &mut mem);

        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_sec_clr() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.set_flag(CARRY_FLAG, false);

        sec(&mut cpu, &mut mem);

        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_sed_set() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.set_flag(DECIMAL_FLAG, true);

        sed(&mut cpu, &mut mem);

        assert!(cpu.get_flag(DECIMAL_FLAG));
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_sed_clr() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.set_flag(DECIMAL_FLAG, false);

        sed(&mut cpu, &mut mem);

        assert!(cpu.get_flag(DECIMAL_FLAG));
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_sei_set() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.set_flag(INTERRUPT_FLAG, true);

        sei(&mut cpu, &mut mem);

        assert!(cpu.get_flag(INTERRUPT_FLAG));
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_sei_clr() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.set_flag(INTERRUPT_FLAG, false);

        sei(&mut cpu, &mut mem);

        assert!(cpu.get_flag(INTERRUPT_FLAG));
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_sta() {
        const ADDR_OPER      : u16 = 0x4567;
        const ACCUMULATOR_VAL:  u8 = 0xf9;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        cpu.a = ACCUMULATOR_VAL;
        cpu.addr_oper = ADDR_OPER;

        sta(&mut cpu, &mut mem);

        assert!(mem.read(ADDR_OPER) == ACCUMULATOR_VAL);
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_stx() {
        const ADDR_OPER: u16 = 0x4567;
        const X_VAL    :  u8 = 0xf9;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        cpu.x = X_VAL;
        cpu.addr_oper = ADDR_OPER;

        stx(&mut cpu, &mut mem);

        assert!(mem.read(ADDR_OPER) == X_VAL);
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_sty() {
        const ADDR_OPER: u16 = 0x4567;
        const Y_VAL    :  u8 = 0xf9;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        cpu.y = Y_VAL;
        cpu.addr_oper = ADDR_OPER;

        sty(&mut cpu, &mut mem);

        assert!(mem.read(ADDR_OPER) == Y_VAL);
        assert!(cpu.cycles == 1);
    }

    #[test]
    fn test_tax() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.a = 0x42;

        tax(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x42);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tax_negative() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.a = 0x80;

        tax(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tax_zero() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.a = 0x00;

        tax(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tay() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.a = 0x42;

        tay(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0x42);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tay_negative() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.a = 0x80;

        tay(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tay_zero() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.a = 0x00;

        tay(&mut cpu, &mut mem);

        assert_eq!(cpu.y, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tsx() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.sp = 0x42;

        tsx(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x42);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tsx_negative() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.sp = 0x80;

        tsx(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tsx_zero() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.sp = 0x00;

        tsx(&mut cpu, &mut mem);

        assert_eq!(cpu.x, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_txa() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x42;

        txa(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x42);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_txa_negative() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x80;

        txa(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_txa_zero() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x00;

        txa(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_txs() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x42;
        cpu.set_flag(NEGATIVE_FLAG, true);
        cpu.set_flag(ZERO_FLAG, true);

        txs(&mut cpu, &mut mem);

        assert_eq!(cpu.sp, 0x42);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_txs_does_not_affect_flags() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.x = 0x80;
        cpu.set_flag(NEGATIVE_FLAG, false);
        cpu.set_flag(ZERO_FLAG, false);

        txs(&mut cpu, &mut mem);

        assert_eq!(cpu.sp, 0x80);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tya() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0x42;

        tya(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x42);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tya_negative() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0x80;

        tya(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x80);
        assert!(cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_tya_zero() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        cpu.y = 0x00;

        tya(&mut cpu, &mut mem);

        assert_eq!(cpu.a, 0x00);
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
        assert_eq!(cpu.cycles, 1);
    }
}
