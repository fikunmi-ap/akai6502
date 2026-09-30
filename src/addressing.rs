//! Addressing Modes
//! 
//! The 6502 has 13 different addressing modes that all do the same thing;
//! compute the address of the operand.
//! 
//! Because of this uniformity, the addressing mode work can be factored out
//! as is done into this module.

use crate::{cpu::CPU, Bus};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum AddressingMode {
    /// Used when the operand is implied by the instruction itself like
    /// `clear_carry_flag`.
    ///
    /// Single byte instruction. Takes 0 cycles.
    Implied,

    /// Operates on the Accumulator (A) register.
    ///
    /// This addressing mode is special in that the operand is the accumulator
    /// itself, which can't be represented as a 16-bit address. Fetching from/writing
    /// to the accumulator must be handled in the operation functions as necessary.
    ///
    /// Single byte instruction. Takes 0 cycles.
    Accumulator,

    /// The operand is inlined directly into the instruction stream. e.g. `LDA #10`.
    /// 
    /// Note that the PC should be incremented as part of the memory read but
    /// that is handled already in `resolve()` for ergonomics
    ///
    /// Two byte instruction. Takes 0 cycles.
    Immediate,

    /// The operand is stored in the zero page (upper byte is zero).
    /// The lower byte (offset) is directly inlined in the instruction stream.
    ///
    /// Two byte instruction. Takes 1 cycle.
    ZeroPage,

    /// Zero page addressing with X register offset.
    /// The operand address is calculated by adding the X register to the 8-bit
    /// zero page address in the instruction stream.
    ///
    /// For example `LDA $10,X` means add $10 to the value in register X and load
    /// the value from that zero page address into the accumulator.
    ///
    /// Note: This is a wrapping addition - the result must remain on the zero page
    /// so it wraps around if it overflows.
    ///
    /// Two byte instruction. Takes 2 cycles.
    ZeroPageX,

    /// Zero page addressing with Y register offset.
    /// The operand address is calculated by adding the Y register to the 8-bit
    /// zero page address in the instruction stream.
    ///
    /// For example `LDA $10,Y` means add $10 to the value in register Y and load
    /// the value from that zero page address into the accumulator.
    /// 
    /// Note: This is a wrapping addition - the result must remain on the zero page
    /// so it wraps around if it overflows.
    ///
    /// Two byte instruction. Takes 2 cycles.
    ZeroPageY,

    /// Used by branch instructions. Contains a signed 8-bit offset (-128 to +127)
    /// that is added to the program counter to calculate the branch target.
    ///
    /// Note: Because the program counter is incremented as part of reading
    /// and decoding the instruction and inlined data, the effective address
    /// range is -126 to +129 bytes from the branch instruction.
    ///
    /// Two byte instruction. Takes 0 cycles (additional cycles handled by branch logic).
    Relative,

    /// The instruction stream contains the full 16-bit address of the operand.
    ///
    /// Three byte instruction. Takes 2 cycles.
    Absolute,

    /// JSR uses absolute addressing but only fetches the low byte during
    /// address resolution. The high byte is fetched after the stack writes.
    ///
    /// This is an internal timing variant, not a separate 6502 addressing
    /// mode.
    AbsoluteJsr,

    /// Absolute addressing with X register offset.
    /// The operand address is calculated by adding the X register to the 16-bit
    /// address in the instruction stream.
    ///
    /// If the final address crosses a page boundary, an additional clock cycle
    /// is required.
    ///
    /// Three byte instruction. Takes 2+ cycles.
    AbsoluteX,

    /// Absolute addressing with Y register offset.
    /// The operand address is calculated by adding the Y register to the 16-bit
    /// address in the instruction stream.
    ///
    /// If the final address crosses a page boundary, an additional clock cycle
    /// is required.
    ///
    /// Three byte instruction. Takes 2+ cycles.
    AbsoluteY,

    /// Indirect addressing mode. The instruction stream contains the address of
    /// a pointer to the actual operand address.
    ///
    /// `JMP` is the only 6502 instruction that supports indirect addressing.
    ///
    /// Note: There is a bug in the 6502 hardware - if the low byte of the
    /// pointer is `0xff` (at a page boundary), the high byte is read from
    /// the beginning of the same page instead of the next page.
    ///
    /// Three byte instruction. Takes 4 cycles.
    Indirect,

    /// Indexed Indirect addressing (also called IndirectX).
    /// The instruction contains a zero page address (single byte) which is 
    /// added to the X register.
    /// 
    /// The result is an index to the zero page and that location contains the
    /// low byte of the target address, with the high byte in the next address.
    ///
    /// This is commonly used with jump tables stored in zero page.
    ///
    /// Two byte instruction. Takes 3 cycles.
    IndexedIndirect,

    /// Indirect Indexed addressing (also called IndirectY).
    /// The instruction contains a zero page address pointing to the low byte of
    /// a base address. The Y register is added to this base address to get the
    /// final operand address.
    ///
    /// This is commonly used with data structures and arrays.
    ///
    /// If the final address crosses a page boundary, an additional clock cycle
    /// is required.
    ///
    /// Two byte instruction. Takes 2+ cycles.
    IndirectIndexed,
}

impl Default for AddressingMode {
    fn default() -> Self {
        // Chosen because it's the simplest addressing mode possible.
        AddressingMode::Implied
    }
}

impl AddressingMode {
    /// Resolve the addressing mode and calculate the operand address.
    pub fn resolve<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
        match cpu.addr_mode {
            AddressingMode::Implied => implied(cpu, bus),
            AddressingMode::Accumulator => accumulator(cpu, bus),
            AddressingMode::Immediate => immediate(cpu, bus),
            AddressingMode::ZeroPage => zero_page(cpu, bus),
            AddressingMode::ZeroPageX => zero_page_x(cpu, bus),
            AddressingMode::ZeroPageY => zero_page_y(cpu, bus),
            AddressingMode::Relative => relative(cpu, bus),
            AddressingMode::Absolute => absolute(cpu, bus),
            AddressingMode::AbsoluteJsr => absolute_jsr(cpu, bus),
            AddressingMode::AbsoluteX => absolute_x(cpu, bus),
            AddressingMode::AbsoluteY => absolute_y(cpu, bus),
            AddressingMode::Indirect => indirect(cpu, bus),
            AddressingMode::IndexedIndirect => indexed_indirect(cpu, bus),
            AddressingMode::IndirectIndexed => indirect_indexed(cpu, bus),
        }
    }
}

fn implied<B: Bus>(cpu: &mut CPU<B>, _bus: &mut B) {
    cpu.extra_cycle = false
}

fn accumulator<B: Bus>(cpu: &mut CPU<B>, _bus: &mut B) {
    cpu.extra_cycle = false
}

fn immediate<B: Bus>(cpu: &mut CPU<B>, _bus: &mut B) {
    cpu.addr_oper = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.extra_cycle = false
}

fn zero_page<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let zero_page_offset = cpu.fetch_from_and_increment_pc(bus);
    cpu.addr_oper = zero_page_offset as u16;
    cpu.extra_cycle = false
}

fn zero_page_x<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let imm = cpu.fetch_from_and_increment_pc(bus);
    cpu.fetch_byte(imm as u16, bus);
    let offset = imm.wrapping_add(cpu.x);
    cpu.addr_oper = offset as u16;
    cpu.extra_cycle = false
}

fn zero_page_y<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let imm = cpu.fetch_from_and_increment_pc(bus);
    cpu.fetch_byte(imm as u16, bus);
    let offset = imm.wrapping_add(cpu.y);
    cpu.addr_oper = offset as u16;
    cpu.extra_cycle = false
}

fn relative<B: Bus>(cpu: &mut CPU<B>, _bus: &mut B) {
    cpu.addr_oper = cpu.pc;
    cpu.pc = cpu.pc.wrapping_add(1);
    cpu.extra_cycle = true;
}

fn absolute<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let addr_lo = cpu.fetch_from_and_increment_pc(bus) as u16;
    let addr_hi = cpu.fetch_from_and_increment_pc(bus) as u16;
    cpu.addr_oper = addr_hi << 8 | addr_lo;
    
}

fn absolute_jsr<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let addr_lo = cpu.fetch_from_and_increment_pc(bus);
    cpu.addr_oper = addr_lo as u16;
    cpu.extra_cycle = false;
}

fn absolute_x<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let addr_lo = cpu.fetch_from_and_increment_pc(bus) as u16;
    let addr_hi = (cpu.fetch_from_and_increment_pc(bus) as u16) << 8;
    let base_addr = addr_hi | addr_lo;
    cpu.addr_oper = base_addr.wrapping_add(cpu.x as u16);
    cpu.addr_oper_prov = addr_hi | (cpu.addr_oper & 0x00ff);
    cpu.extra_cycle = (cpu.addr_oper >> 8) != (base_addr >> 8);
}

fn absolute_y<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let addr_lo = cpu.fetch_from_and_increment_pc(bus) as u16;
    let addr_hi = cpu.fetch_from_and_increment_pc(bus) as u16;
    let base_addr = addr_hi << 8 | addr_lo;
    cpu.addr_oper = base_addr.wrapping_add(cpu.y as u16);
    cpu.addr_oper_prov = (addr_hi << 8) | (cpu.addr_oper & 0x00ff);
    cpu.extra_cycle = (cpu.addr_oper >> 8) != (base_addr >> 8);
}

fn indirect<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let ptr_lo = cpu.fetch_from_and_increment_pc(bus);
    let ptr_hi = cpu.fetch_from_and_increment_pc(bus);
    let ptr = (ptr_hi as u16) << 8 | ptr_lo as u16;

    if ptr_lo == 0xff {
        let addr_lo = cpu.fetch_byte(ptr, bus);
        let addr_hi = cpu.fetch_byte((ptr_hi as u16) << 8, bus);
        cpu.addr_oper = (addr_hi as u16) << 8 | addr_lo as u16;
    } else {
        let addr_lo = cpu.fetch_byte(ptr, bus);
        let addr_hi = cpu.fetch_byte(ptr + 1, bus);
        cpu.addr_oper = (addr_hi as u16) << 8 | addr_lo as u16;
    }
}

fn indexed_indirect<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let zp_addr = cpu.fetch_from_and_increment_pc(bus);
    cpu.fetch_byte(zp_addr as u16, bus);
    let zp_offset = zp_addr.wrapping_add(cpu.x);
    let addr_lo = cpu.fetch_byte(zp_offset as u16, bus);
    let addr_hi = cpu.fetch_byte((zp_offset.wrapping_add(1)) as u16, bus);
    cpu.addr_oper = (addr_hi as u16) << 8 | addr_lo as u16;
}

fn indirect_indexed<B: Bus>(cpu: &mut CPU<B>, bus: &mut B) {
    let zp_offset = cpu.fetch_from_and_increment_pc(bus);
    let addr_lo = cpu.fetch_byte(zp_offset as u16, bus);
    let addr_hi = cpu.fetch_byte((zp_offset.wrapping_add(1)) as u16, bus);
    let base_addr = (addr_hi as u16) << 8 | addr_lo as u16;
    cpu.addr_oper = base_addr.wrapping_add(cpu.y as u16);
    cpu.addr_oper_prov = (base_addr & 0xff00) | (cpu.addr_oper & 0x00ff);
    cpu.extra_cycle = (cpu.addr_oper >> 8) != (base_addr >> 8);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bus::setup_cpu_and_memory;

    #[test]
    fn test_zero_page() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        mem.write(0x8000, 0x42);
        
        cpu.pc = 0x8000;
        zero_page(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x0042);
        assert_eq!(cpu.pc, 0x8001);
    }

    #[test]
    fn test_zero_page_x() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        mem.write(0x8000, 0x20);

        cpu.pc = 0x8000;
        cpu.x = 0x05;
        
        zero_page_x(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x0025);
        assert_eq!(cpu.pc, 0x8001);
    }

    #[test]
    fn test_zero_page_x_wrapping() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        mem.write(0x8000, 0x20);

        cpu.pc = 0x8000;
        cpu.x = 0xff;
        
        zero_page_x(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x001f);
        assert_eq!(cpu.pc, 0x8001);
    }

    #[test]
    fn test_zero_page_y() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        cpu.pc = 0x8000;
        cpu.y = 0x03;
        mem.write(0x8000, 0x20);
        
        zero_page_y(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x0023);
        assert_eq!(cpu.pc, 0x8001);
    }

    #[test]
    fn test_zero_page_y_wrapping() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        mem.write(0x8000, 0x20);

        cpu.pc = 0x8000;
        cpu.y = 0xff;
        
        zero_page_y(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x001f);
        assert_eq!(cpu.pc, 0x8001);
    }

    #[test]
    fn test_relative() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        cpu.pc = 0x8000;
        
        relative(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x8000);
        assert_eq!(cpu.pc, 0x8001);
        assert!(cpu.extra_cycle);
    }

    #[test]
    fn test_absolute() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        mem.write(0x8000, 0x34);  // Low byte
        mem.write(0x8001, 0x12);  // High byte
        
        cpu.pc = 0x8000;
        
        absolute(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x1234);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_absolute_jsr() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        mem.write(0x8000, 0x42);
        cpu.pc = 0x8000;

        absolute_jsr(&mut cpu, &mut mem);

        assert_eq!(cpu.addr_oper, 0x0042);
        assert_eq!(cpu.pc, 0x8001);
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_absolute_x_same_page() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        mem.write(0x8000, 0x00);  // Low byte
        mem.write(0x8001, 0x30);  // High byte

        cpu.pc = 0x8000;
        cpu.x = 0x05;
        
        absolute_x(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x3005);
        assert_eq!(cpu.pc, 0x8002);
        assert!(!cpu.extra_cycle);
    }

    #[test]
    fn test_absolute_x_different_page() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        cpu.pc = 0x8000;
        cpu.x = 0xff;
        mem.write(0x8000, 0x02);  // Low byte
        mem.write(0x8001, 0x30);  // High byte
        
        absolute_x(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x3101);
        assert_eq!(cpu.pc, 0x8002);
        assert!(cpu.extra_cycle);
    }

    #[test]
    fn test_absolute_y_same_page() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();  
        
        cpu.pc = 0x8000;
        cpu.y = 0x05;
        mem.write(0x8000, 0x00);  // Low byte
        mem.write(0x8001, 0x30);  // High byte
        
        absolute_y(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x3005);
        assert_eq!(cpu.pc, 0x8002);
        assert!(!cpu.extra_cycle);
    }

    #[test]
    fn test_absolute_y_different_page() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        
        cpu.pc = 0x8000;
        cpu.y = 0xff;
        mem.write(0x8000, 0x02);  // Low byte
        mem.write(0x8001, 0x30);  // High byte
        
        absolute_y(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x3101);
        assert_eq!(cpu.pc, 0x8002);
        assert!(cpu.extra_cycle);
    }

    #[test]
    fn test_indirect_non_boundary() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        mem.write(0x8000, 0x20);  // Pointer low byte
        mem.write(0x8001, 0x30);  // Pointer high byte
        mem.write(0x3020, 0x34);  // Target address low byte
        mem.write(0x3021, 0x12);  // Target address high byte
        
        cpu.pc = 0x8000;

        indirect(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x1234);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_indirect_at_boundary() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        cpu.pc = 0x8000;
        mem.write(0x8000, 0xff);  // Pointer low byte (at page boundary)
        mem.write(0x8001, 0x30);  // Pointer high byte
        mem.write(0x30ff, 0x34);  // Target address low byte
        mem.write(0x3000, 0x12);  // Target address high byte (wraps to page start)
        
        indirect(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x1234);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_indexed_indirect() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        cpu.pc = 0x8000;
        cpu.x = 0x04;

        mem.write(0x8000, 0x20);  // Zero page base
        mem.write(0x0024, 0x34);  // Target address low byte (0x20 + 0x04)
        mem.write(0x0025, 0x12);  // Target address high byte
        
        indexed_indirect(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x1234);
        assert_eq!(cpu.pc, 0x8001);
    }

    #[test]
    fn test_indirect_indexed_same_page() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        cpu.pc = 0x8000;
        cpu.y = 0x10;
        mem.write(0x8000, 0x20);  // Zero page offset
        mem.write(0x0020, 0x00);  // Base address low byte
        mem.write(0x0021, 0x30);  // Base address high byte
        
        indirect_indexed(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x3010);
        assert_eq!(cpu.pc, 0x8001);
        assert!(!cpu.extra_cycle);
    }

    #[test]
    fn test_indirect_indexed_different_page() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        
        cpu.pc = 0x8000;
        cpu.y = 0xff;
        mem.write(0x8000, 0x20);  // Zero page offset
        mem.write(0x0020, 0x02);  // Base address low byte
        mem.write(0x0021, 0x30);  // Base address high byte
        
        indirect_indexed(&mut cpu, &mut mem);
        
        assert_eq!(cpu.addr_oper, 0x3101);
        assert_eq!(cpu.pc, 0x8001);
        assert!(cpu.extra_cycle);
    }
}
