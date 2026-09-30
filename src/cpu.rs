//! CPU core

use std::fmt;

use crate::{
    BREAK_FLAG, Bus, Byte, DMATransferStatus, INTERRUPT_FLAG, IRQ_PENDING, IRQ_VECTOR_HI,
    IRQ_VECTOR_LO, NMI_PENDING, NMI_VECTOR_HI, NMI_VECTOR_LO, RESET_PENDING, RESET_VECTOR_HI,
    RESET_VECTOR_LO, UNUSED_FLAG, Word, addressing::AddressingMode,
    instructions::InstructionHandler, opcodes::Opcode,
};

// TODO: Complete documentation here.
/// Internal representation of the CPU state.
///
/// - 16 bit Program Counter
/// - 8 bit Stack Pointer
/// - 8 bit registers A, X, and Y.
/// - 8 bit status register.
///
/// If we were going for a compile time representation that matches the domain
/// model, we would stop there, but I'll add additional stuff:
/// - state:
/// - addr_abs: The absolute address of the data that the instruction wants to
/// work on so each instruction can call `fetch_data` in its execution and
/// call it a day.
#[derive(Debug)]
pub struct CPU<B: Bus> {
    /// General purpose 8 bit register called the accumulator that is used in
    /// arithmetic and logical operations.
    pub(crate) a: Byte,

    /// General purpose 8 bit register, most commonly used to hold counters, or
    /// offsets.
    ///
    /// Can be used to get a copy of or change the stack pointer.
    pub(crate) x: Byte,

    /// General purpose 8 bit register, most commonly used to hold counters, or
    /// offsets.
    pub(crate) y: Byte,

    /// Holds 8 1-bit flags that show the result of the operation.
    /// The layout of the status register flags is:
    ///
    /// | NEGATIVE| OVERFLOW| UNUSED| BREAK| DECIMAL| INTERRUPT| ZERO| CARRY |
    pub(crate) sr: Byte,

    /// 8-bit register that holds the low 8-bits of the next free location on
    /// the stack.
    ///
    /// The stack on the 6502 is between 0x0100 and 0x01ff but the stack pointer
    /// saves space by not storing the page i.e. the higher 8 bits.
    pub(crate) sp: Byte,

    /// 16 bit program counter that points to the next instruction to be
    /// executed.
    pub(crate) pc: Word,

    /// Number of cycles run since instantiation.
    pub(crate) cycles: u64,

    // -------------------- IMPLEMENTATION DETAILS ------------------------- //
    pub(crate) state: CPUState,

    /// Opcode currently being executed.
    pub(crate) opcode: Byte,

    pub(crate) addr_mode: AddressingMode,

    /// Some addressing mode--operation combos require an additional cycle.
    ///
    /// This bool is initially set during the addressing mode function and
    /// asserted true or false during the operation.
    pub(crate) extra_cycle: bool,

    /// Absolute address of the operand that the operation being executed needs.
    /// Calculated in different ways for different addressing modes.
    pub(crate) addr_oper: Word,

    /// Address placed on the bus before an indexed address is fully corrected.
    ///
    /// Indexed stores always read from this address before writing to
    /// `addr_oper`. Indexed reads use it when a page crossing occurs.
    pub(crate) addr_oper_prov: Word,

    /// The operand fetched from `addr_abs`.
    pub(crate) operand: Byte,

    // Interrupt Handling
    /// Holds 3 1-bit flags to indicate when:
    /// - NMI_PENDING
    /// - IRQ_PENDING
    /// - RESET_PENDING
    pub(crate) interrupts: Byte,

    /// Previous NMI line state, used to detect rising edges.
    pub(crate) nmi_line_high: bool,

    /// Whether the IRQ poll at the end of the previous instruction accepted
    /// an interrupt for the next instruction boundary.
    pub(crate) irq_accepted: bool,

    /// Value of the I flag that governs the current instruction's IRQ poll.
    pub(crate) irq_poll_i_flag: bool,

    /// Whether `irq_accepted` contains a decision made by a completed poll.
    pub(crate) irq_poll_valid: bool,

    // DMA Handling
    pub(crate) dma_active: bool,

    /// Mode
    pub(crate) mode: CPUMode,

    /// OPCode table.
    ///
    /// This is necessary because each implementation of the Bus trait requires
    /// its own OPCODE table (because no generics in static/const definitions)
    pub(crate) opcode_table: [InstructionHandler<B>; 256],
}

/// CPU state machine.
#[derive(Debug, PartialEq)]
pub enum CPUState {
    FetchOpcode,
    AddressModeResolution,
    Execute,
}

/// CPU Modes
///
/// Each one enables a different feature set.
#[derive(Debug, PartialEq)]
pub enum CPUMode {
    Normal,
    Debug,
    KlausDormann,
}

impl<B: Bus> CPU<B> {
    pub fn new() -> CPU<B> {
        let opcode_table = crate::instructions::create_opcode_table();

        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sr: 0b0000_0000,
            sp: 0xff,
            pc: 0xff_fc,
            cycles: 0,
            state: CPUState::FetchOpcode,
            opcode: 0x00,
            addr_mode: AddressingMode::default(),
            extra_cycle: false,
            addr_oper: 0x00_00,
            addr_oper_prov: 0x00_00,
            operand: 0x00,
            interrupts: 0b0000_0000,
            nmi_line_high: false,
            irq_accepted: false,
            irq_poll_i_flag: false,
            irq_poll_valid: false,
            dma_active: false,
            mode: CPUMode::Normal,
            opcode_table,
        }
    }

    pub fn new_debug() -> CPU<B> {
        let opcode_table = crate::instructions::create_opcode_table();

        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sr: 0b0000_0000,
            sp: 0xff,
            pc: 0xff_fc,
            cycles: 0,
            state: CPUState::FetchOpcode,
            opcode: 0x00,
            addr_mode: AddressingMode::default(),
            extra_cycle: false,
            addr_oper: 0x00_00,
            addr_oper_prov: 0x00_00,
            operand: 0x00,
            interrupts: 0b0000_0000,
            nmi_line_high: false,
            irq_accepted: false,
            irq_poll_i_flag: false,
            irq_poll_valid: false,
            dma_active: false,
            mode: CPUMode::Debug,
            opcode_table,
        }
    }

    pub fn new_dormann() -> CPU<B> {
        let opcode_table = crate::instructions::create_opcode_table();
        Self {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            sr: 0b0000_0000,
            sp: 0xff,
            pc: 0x0400,
            cycles: 0,
            state: CPUState::FetchOpcode,
            opcode: 0x00,
            addr_mode: AddressingMode::default(),
            extra_cycle: false,
            addr_oper: 0x00_00,
            addr_oper_prov: 0x00_00,
            operand: 0x00,
            interrupts: 0b0000_0000,
            nmi_line_high: false,
            irq_accepted: false,
            irq_poll_i_flag: false,
            irq_poll_valid: false,
            dma_active: false,
            mode: CPUMode::KlausDormann,
            opcode_table,
        }
    }

    /// Continuously executes instructions until a self jump is detected i.e.,
    /// the instruction sets the PC to the location it was at before the
    /// instruction executed which creates an infinite loop.
    ///
    /// Prints the CPU state at the end of every instruction.
    pub fn run(&mut self, bus: &mut B) {
        loop {
            let pc_before = self.pc;
            self.execute_single_instruction(bus);
            if self.pc == pc_before {
                break;
            }
        }
    }

    /// Fetch the next opcode from the current PC, decode it, resolve the
    /// addressing mode, and execute the transaction.
    pub fn execute_single_instruction(&mut self, bus: &mut B) {
        // Keep the external line states current. If no instruction has
        // produced an IRQ poll yet, treat this as the initial boundary.
        self.sample_interrupts(bus);
        if !self.irq_poll_valid {
            self.refresh_irq_poll_i_flag();
            self.latch_irq_poll_result();
        }

        if self.service_interrupts(bus) {
            self.irq_accepted = false;
            self.irq_poll_valid = false;
            return;
        }

        // CLI, SEI, and PLP change I too late to affect this instruction's
        // poll, so capture I before the instruction. RTI updates this snapshot
        // after restoring the status register.
        self.refresh_irq_poll_i_flag();

        // PC before it's incremented as part of instruction fetching.
        // A good compiler will remove this in release builds.
        let instruction_pc = self.pc;

        self.fetch_opcode(bus);
        self.resolve_addr_mode(bus);
        self.execute(bus);
        self.latch_irq_poll_result();
        // Print the instruction details.
        #[cfg(debug_assertions)]
        {
            println!(
                "PC: ${:04X} | {} | OPER: {:02X} │ A: {:02X} │ X: {:02X} │ Y: {:02X} │ SP: ${:02X} │ FLAGS: [{}{}{}{}{}{}{}{}]",
                instruction_pc,
                Opcode::from_byte(self.opcode),
                self.operand,
                self.a,
                self.x,
                self.y,
                self.sp,
                if self.sr & 0b1000_0000 != 0 { 'N' } else { '-' },
                if self.sr & 0b0100_0000 != 0 { 'V' } else { '-' },
                if self.sr & 0b0010_0000 != 0 { '1' } else { '-' },
                if self.sr & 0b0001_0000 != 0 { 'B' } else { '-' },
                if self.sr & 0b0000_1000 != 0 { 'D' } else { '-' },
                if self.sr & 0b0000_0100 != 0 { 'I' } else { '-' },
                if self.sr & 0b0000_0010 != 0 { 'Z' } else { '-' },
                if self.sr & 0b0000_0001 != 0 { 'C' } else { '-' },
            );
        }
    }

    /// Get the next instruction opcode
    fn fetch_opcode(&mut self, bus: &mut B) {
        self.opcode = self.fetch_from_and_increment_pc(bus);
        // Get addressing mode from opcode table
        self.addr_mode = self.opcode_table[self.opcode as usize].addr_mode;
        self.state = CPUState::AddressModeResolution;
    }

    /// Resolve the addr_oper for the instruction
    fn resolve_addr_mode(&mut self, bus: &mut B) {
        AddressingMode::resolve(self, bus);
        self.state = CPUState::Execute;
    }

    /// Execute cycle: Run the instruction
    pub(crate) fn execute(&mut self, bus: &mut B) {
        let instruction = &self.opcode_table[self.opcode as usize];
        (instruction.operation)(self, bus);
        self.state = CPUState::FetchOpcode
    }

    /// Perform all end-of-cycle checks and updates.
    ///
    /// This should be called at the end of every CPU cycle.
    fn end_of_cycle(&mut self, bus: &mut B) {
        self.cycles += 1;
        bus.cycle();
        self.sample_interrupts(bus);
        // self.check_debug_conditions();
    }

    /// Sample the external interrupt lines.
    ///
    /// IRQ acceptance is a separate instruction-boundary poll because the
    /// value of I used by that poll is instruction-timing dependent.
    ///
    /// TODO: Check that these if/else branches make sense based on the
    /// constraints of emulation.
    fn sample_interrupts(&mut self, bus: &B) {
        // RESET is level triggered and has the highest priority.
        if bus.check_reset() {
            if !self.get_interrupt_flag(RESET_PENDING) {
                self.set_interrupt_flag(RESET_PENDING, true);
            }
        }

        // NMI is edge-triggered and has the next highest priority.
        // But it should only latch on a rising edge.
        let nmi_line_high = bus.check_nmi();
        if nmi_line_high && !self.nmi_line_high {
            self.set_interrupt_flag(NMI_PENDING, true);
        }
        self.nmi_line_high = nmi_line_high;

        // IRQ is level-triggered. The I flag controls whether IRQ is serviced,
        // not whether the current state of the line is recorded.
        self.set_interrupt_flag(IRQ_PENDING, bus.check_irq());
    }

    /// Capture the current I flag for the current instruction's IRQ poll.
    pub(crate) fn refresh_irq_poll_i_flag(&mut self) {
        self.irq_poll_i_flag = self.get_flag(INTERRUPT_FLAG);
    }

    /// Latch whether IRQ should be serviced at the next instruction boundary.
    fn latch_irq_poll_result(&mut self) {
        self.irq_accepted = self.get_interrupt_flag(IRQ_PENDING) && !self.irq_poll_i_flag;
        self.irq_poll_valid = true;
    }

    /// Service the highest priority interrupt. Should be called at instruction
    /// boundaries.
    fn service_interrupts(&mut self, bus: &mut B) -> bool {
        if self.get_interrupt_flag(RESET_PENDING) {
            self.reset(bus);
        } else if self.get_interrupt_flag(NMI_PENDING) {
            self.nmi(bus);
        } else if self.irq_accepted {
            self.irq(bus);
        } else {
            return false;
        }
        true
    }

    /// Handle a reset,
    fn reset(&mut self, bus: &mut B) {
        // Read PC twice.
        self.dummy_read(bus);
        self.dummy_read(bus);

        // Read and decrement the stack-pointer address three times. Nothing is
        // actually pushed, so memory remains unchanged.
        for _ in 0..3 {
            self.fetch_byte(self.full_stack_pointer_address(), bus);
            self.sp = self.sp.wrapping_sub(1);
        }

        self.set_flag(INTERRUPT_FLAG, true);

        // Set PC to the reset vector.
        let reset_vector_lo = self.fetch_byte(RESET_VECTOR_LO, bus);
        let reset_vector_hi = self.fetch_byte(RESET_VECTOR_HI, bus);
        self.pc = (reset_vector_hi as Word) << 8 | reset_vector_lo as Word;

        self.state = CPUState::FetchOpcode;
        self.set_interrupt_flag(RESET_PENDING, false);
    }

    /// Handle a Non Maskable Interrupt (NMI)
    ///
    /// Takes seven cycles.
    fn nmi(&mut self, bus: &mut B) {
        // todo: ask why setting the flag to false.
        self.set_interrupt_flag(NMI_PENDING, false);

        // Read PC twice
        self.dummy_read(bus);
        self.dummy_read(bus);
        // Push PC to stack
        self.push_word_to_stack(bus, self.pc);
        // Push status reg with B flag set to 0 and U flag set to 1 to stack.
        let status = (self.sr & !BREAK_FLAG) | UNUSED_FLAG;
        self.push_byte_to_stack(bus, status);
        self.set_flag(INTERRUPT_FLAG, true);

        // Set PC to interrupt handler
        let nmi_handler_ptr_lo = self.fetch_byte(NMI_VECTOR_LO, bus);
        let nmi_handler_ptr_hi = self.fetch_byte(NMI_VECTOR_HI, bus);
        self.pc = (nmi_handler_ptr_hi as Word) << 8 | nmi_handler_ptr_lo as Word;
        self.state = CPUState::FetchOpcode;
    }

    fn irq(&mut self, bus: &mut B) {
        self.irq_accepted = false;
        self.set_interrupt_flag(IRQ_PENDING, false);
        // Read PC twice
        self.dummy_read(bus);
        self.dummy_read(bus);
        // Push PC to stack
        self.push_word_to_stack(bus, self.pc);
        // Push status reg with B flag set to 0 and U flag set to 1 to stack.
        let status = (self.sr & !BREAK_FLAG) | UNUSED_FLAG;
        self.push_byte_to_stack(bus, status);
        self.set_flag(INTERRUPT_FLAG, true);

        // jump to interrupt handler
        let irq_handler_ptr_lo = self.fetch_byte(IRQ_VECTOR_LO, bus);
        let irq_handler_ptr_hi = self.fetch_byte(IRQ_VECTOR_HI, bus);
        self.pc = (irq_handler_ptr_hi as Word) << 8 | irq_handler_ptr_lo as Word;
        self.state = CPUState::FetchOpcode;
    }

    /// Check if an interrupt flag is set.
    fn get_interrupt_flag(&self, flag: u8) -> bool {
        (self.interrupts & flag) != 0
    }

    /// Helper functions for interrupt flag manipulation
    fn set_interrupt_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.interrupts |= flag;
        } else {
            self.interrupts &= !flag;
        }
    }

    /// Checks if there is an unserviced or in-service DMA request and if this
    /// is the cycle to handle it.
    fn should_handle_dma(&self, bus: &B) -> bool {
        bus.check_dma_request() && (self.cycles % 2 == 0)
    }

    /// Handle DMA (Direct Memory Access) requests
    /// DMA can steal CPU cycles, affecting timing accuracy
    fn handle_dma_requests(&mut self, bus: &mut B) {
        if let Some(channel) = bus.get_active_dma_channel() {
            let transfer_status = bus.perform_dma_transfer(channel);

            match transfer_status {
                DMATransferStatus::Complete | DMATransferStatus::Aborted => {
                    self.dma_active = false;
                }
                DMATransferStatus::InProgress { .. } => {
                    todo!()
                }
            }
        }
        self.end_of_cycle(bus);
    }

    /// Check for debugging conditions like breakpoints
    fn check_debug_conditions(&mut self) {
        todo!()
    }

    /// Set a single or multiple flags in the status register to `value`
    ///
    /// Takes a bitmask of the flags to set as input.
    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            // if set to true, just bitwise or the flag with the current status.
            self.sr |= flag;
        } else {
            // if set to 0, perform a bitwise and of the current status and the
            // bitwise not of the flag constant (which will be 0 for the bit
            // but 1 everywhere else)
            self.sr &= !flag;
        }
    }

    /// Get the state of a flag in the status register.
    ///
    /// Takes advantage of the fact that applying a bitwise AND to the current state of
    /// the register with the flag constant will be 1 iff the flag is set and 0
    /// otherwise.
    pub fn get_flag(&self, flag: u8) -> bool {
        (self.sr & flag) != 0
    }

    /// Fetches the byte from wherever the program counter is pointing and
    /// increments the program counter.
    ///
    /// Can be used to fetch the next opcode or data depending on the
    /// situation.
    ///
    /// Takes one cycle.
    pub(crate) fn fetch_from_and_increment_pc(&mut self, bus: &mut B) -> Byte {
        let byte = self.fetch_byte(self.pc, bus);
        // This is incremented by 1 because our memory is byte addressable.
        // It's literally an array of bytes:
        // Index 0 gives you a byte.
        // Index 1 gives you the next byte.
        // And so on.
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    /// Reads and returns the byte at the provided address.
    ///
    /// Takes 1 cycle.
    pub(crate) fn fetch_byte(&mut self, address: Word, bus: &mut B) -> Byte {
        let byte = bus.read(address);
        self.end_of_cycle(bus);
        byte
    }

    /// Writes value to the byte at the provided address.
    ///
    /// Takes 1 cycle
    pub(crate) fn write_byte(&mut self, address: Word, data: Byte, bus: &mut B) {
        bus.write(address, data);
        self.end_of_cycle(bus);
    }

    /// Fetch the word whose LSB's address is currently in the PC.
    ///
    /// Note that the 6502 is little endian as most machines are today
    /// So this is designed with that in mind. A more complete implementation
    /// would have some way to handle this. A `swap_bytes_in_word` function
    /// would help.
    ///
    /// Takes 2 cycles.
    pub(crate) fn fetch_pc_word(&mut self, bus: &mut B) -> Word {
        let low_byte = self.fetch_byte(self.pc, bus);
        self.pc = self.pc.wrapping_add(1);

        let high_byte = self.fetch_byte(self.pc, bus);
        self.pc = self.pc.wrapping_add(1);

        (high_byte as u16) << 8 | low_byte as u16
    }

    /// The stack pointer only holds the lower 8 bits of the next free location
    /// on the stack because the stack is always between 0x100 and 0x1ff.
    /// You can save loading a byte by removing the higher byte.
    ///
    /// This function is a convenience function to return the full address
    /// when needed.
    pub fn full_stack_pointer_address(&self) -> Word {
        0x01_00 | self.sp as Word
    }

    /// Push one byte to the stack.
    /// Takes one cycle.
    pub(crate) fn push_byte_to_stack(&mut self, bus: &mut B, data: Byte) {
        self.write_byte(self.full_stack_pointer_address(), data, bus);
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Pushes a word to the stack.
    ///
    /// Note that since the stack grows downward, the high byte is pushed first.
    ///
    /// Takes 2 cycles.
    pub(crate) fn push_word_to_stack(&mut self, bus: &mut B, word: Word) {
        self.push_byte_to_stack(bus, (word >> 8) as Byte);
        self.push_byte_to_stack(bus, word as Byte);
    }

    /// Pop a byte from the stack.
    ///
    /// Increments the SP to point to the last byte pushed, then reads it.
    ///
    /// Takes one cycle.
    pub(crate) fn pop_byte_from_stack(&mut self, bus: &mut B) -> Byte {
        self.sp = self.sp.wrapping_add(1);
        let byte = self.fetch_byte(self.full_stack_pointer_address(), bus);
        byte
    }

    /// Read from the current stack address without modifying the SP or using
    /// the value.
    pub(crate) fn dummy_stack_read(&mut self, bus: &mut B) {
        self.fetch_byte(self.full_stack_pointer_address(), bus);
    }

    /// Fetch the operand from the addr_abs
    pub(crate) fn fetch_operand(&mut self, bus: &mut B) {
        self.operand = self.fetch_byte(self.addr_oper, bus)
    }

    /// Read from the current PC and do nothing with the data.
    pub(crate) fn dummy_read(&mut self, bus: &mut B) {
        self.fetch_byte(self.pc, bus);
    }

    // Getters and setters
    pub fn pc(&self) -> Word {
        self.pc
    }

    pub fn set_pc(&mut self, address: Word) {
        self.pc = address
    }

    pub fn opcode(&self) -> Opcode {
        Opcode::from_byte(self.opcode)
    }

    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    pub fn set_addr_oper(&mut self, addr: u16) {
        self.addr_oper = addr
    }
}

impl<B: Bus> fmt::Display for CPU<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PC: ${:04X} | {} {:003} │ A: ${:02X} │ X: ${:02X} │ Y: ${:02X} │ SP: ${:02X} │ FLAGS: [{}{}{}{}{}{}{}{}]",
            self.pc,
            Opcode::from_byte(self.opcode),
            self.operand,
            self.a,
            self.x,
            self.y,
            self.sp,
            if self.sr & 0b1000_0000 != 0 { 'N' } else { '-' },
            if self.sr & 0b0100_0000 != 0 { 'V' } else { '-' },
            if self.sr & 0b0010_0000 != 0 { '1' } else { '-' },
            if self.sr & 0b0001_0000 != 0 { 'B' } else { '-' },
            if self.sr & 0b0000_1000 != 0 { 'D' } else { '-' },
            if self.sr & 0b0000_0100 != 0 { 'I' } else { '-' },
            if self.sr & 0b0000_0010 != 0 { 'Z' } else { '-' },
            if self.sr & 0b0000_0001 != 0 { 'C' } else { '-' },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bus::{BusOperation, TestBus, setup_cpu_and_memory};
    use crate::{
        BREAK_FLAG, CARRY_FLAG, DECIMAL_FLAG, INTERRUPT_FLAG, NEGATIVE_FLAG, OVERFLOW_FLAG,
        UNUSED_FLAG, ZERO_FLAG,
    };

    const TRACE_PC: u16 = 0x8000;

    fn read(address: u16, data: u8) -> BusOperation {
        BusOperation::Read { address, data }
    }

    fn write(address: u16, data: u8) -> BusOperation {
        BusOperation::Write { address, data }
    }

    fn execute_instruction_with_trace(
        cpu: &mut CPU<TestBus>,
        bus: &mut TestBus,
        instruction: &[u8],
    ) -> Vec<BusOperation> {
        execute_instruction_at_with_trace(cpu, bus, TRACE_PC, instruction)
    }

    fn execute_instruction_at_with_trace(
        cpu: &mut CPU<TestBus>,
        bus: &mut TestBus,
        instruction_pc: u16,
        instruction: &[u8],
    ) -> Vec<BusOperation> {
        cpu.pc = instruction_pc;
        for (offset, byte) in instruction.iter().copied().enumerate() {
            bus.write(instruction_pc.wrapping_add(offset as u16), byte);
        }

        bus.clear_bus_operations();
        let initial_cycles = cpu.cycles();
        cpu.execute_single_instruction(bus);
        let operations = bus.bus_operations();

        assert_eq!(
            cpu.cycles(),
            initial_cycles + operations.len() as u64,
            "every traced bus operation must consume one cycle"
        );

        operations
    }

    #[test]
    fn test_asl_zero_page_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x0042, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::AslZpg as u8, 0x42],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::AslZpg as u8),
                read(TRACE_PC + 1, 0x42),
                read(0x0042, 0x81),
                write(0x0042, 0x81),
                write(0x0042, 0x02),
            ]
        );
    }

    #[test]
    fn test_lsr_zero_page_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x0042, 0x03);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LsrZpg as u8, 0x42],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LsrZpg as u8),
                read(TRACE_PC + 1, 0x42),
                read(0x0042, 0x03),
                write(0x0042, 0x03),
                write(0x0042, 0x01),
            ]
        );
    }

    #[test]
    fn test_rol_zero_page_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.set_flag(CARRY_FLAG, true);
        bus.write(0x0042, 0x80);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::RolZpg as u8, 0x42],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::RolZpg as u8),
                read(TRACE_PC + 1, 0x42),
                read(0x0042, 0x80),
                write(0x0042, 0x80),
                write(0x0042, 0x01),
            ]
        );
    }

    #[test]
    fn test_ror_zero_page_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.set_flag(CARRY_FLAG, true);
        bus.write(0x0042, 0x01);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::RorZpg as u8, 0x42],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::RorZpg as u8),
                read(TRACE_PC + 1, 0x42),
                read(0x0042, 0x01),
                write(0x0042, 0x01),
                write(0x0042, 0x80),
            ]
        );
    }

    #[test]
    fn test_inc_zero_page_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x0042, 0xff);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::IncZpg as u8, 0x42],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::IncZpg as u8),
                read(TRACE_PC + 1, 0x42),
                read(0x0042, 0xff),
                write(0x0042, 0xff),
                write(0x0042, 0x00),
            ]
        );
    }

    #[test]
    fn test_dec_zero_page_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x0042, 0x00);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::DecZpg as u8, 0x42],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::DecZpg as u8),
                read(TRACE_PC + 1, 0x42),
                read(0x0042, 0x00),
                write(0x0042, 0x00),
                write(0x0042, 0xff),
            ]
        );
    }

    #[test]
    fn test_asl_zero_page_x_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x20;
        bus.write(0x00f0, 0xa0);
        bus.write(0x0010, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::AslZpx as u8, 0xf0],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::AslZpx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(0x00f0, 0xa0),
                read(0x0010, 0x81),
                write(0x0010, 0x81),
                write(0x0010, 0x02),
            ]
        );
    }

    #[test]
    fn test_asl_absolute_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x1234, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::AslAbs as u8, 0x34, 0x12],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::AslAbs as u8),
                read(TRACE_PC + 1, 0x34),
                read(TRACE_PC + 2, 0x12),
                read(0x1234, 0x81),
                write(0x1234, 0x81),
                write(0x1234, 0x02),
            ]
        );
    }

    #[test]
    fn test_asl_absolute_x_bus_trace_without_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x0f;
        bus.write(0x12ff, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::AslAbx as u8, 0xf0, 0x12],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::AslAbx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(TRACE_PC + 2, 0x12),
                read(0x12ff, 0x81),
                read(0x12ff, 0x81),
                write(0x12ff, 0x81),
                write(0x12ff, 0x02),
            ]
        );
    }

    #[test]
    fn test_asl_absolute_x_bus_trace_with_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x20;
        bus.write(0x1210, 0xa1);
        bus.write(0x1310, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::AslAbx as u8, 0xf0, 0x12],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::AslAbx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(TRACE_PC + 2, 0x12),
                read(0x1210, 0xa1),
                read(0x1310, 0x81),
                write(0x1310, 0x81),
                write(0x1310, 0x02),
            ]
        );
    }

    #[test]
    fn test_asl_accumulator_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x81;
        bus.write(TRACE_PC + 1, 0xa2);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::AslImp as u8],
        );

        assert_eq!(cpu.a, 0x02);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::AslImp as u8),
                read(TRACE_PC + 1, 0xa2),
            ]
        );
    }

    #[test]
    fn test_lsr_accumulator_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x03;
        bus.write(TRACE_PC + 1, 0xa3);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LsrImp as u8],
        );

        assert_eq!(cpu.a, 0x01);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LsrImp as u8),
                read(TRACE_PC + 1, 0xa3),
            ]
        );
    }

    #[test]
    fn test_rol_accumulator_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x80;
        cpu.set_flag(CARRY_FLAG, true);
        bus.write(TRACE_PC + 1, 0xa4);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::RolImp as u8],
        );

        assert_eq!(cpu.a, 0x01);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::RolImp as u8),
                read(TRACE_PC + 1, 0xa4),
            ]
        );
    }

    #[test]
    fn test_ror_accumulator_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x01;
        cpu.set_flag(CARRY_FLAG, true);
        bus.write(TRACE_PC + 1, 0xa5);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::RorImp as u8],
        );

        assert_eq!(cpu.a, 0x80);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::RorImp as u8),
                read(TRACE_PC + 1, 0xa5),
            ]
        );
    }

    #[test]
    fn test_implied_instruction_bus_traces() {
        for opcode in [
            Opcode::ClcImp,
            Opcode::CldImp,
            Opcode::CliImp,
            Opcode::ClvImp,
            Opcode::DexImp,
            Opcode::DeyImp,
            Opcode::InxImp,
            Opcode::InyImp,
            Opcode::NopImp,
            Opcode::SecImp,
            Opcode::SedImp,
            Opcode::SeiImp,
            Opcode::TaxImp,
            Opcode::TayImp,
            Opcode::TsxImp,
            Opcode::TxaImp,
            Opcode::TxsImp,
            Opcode::TyaImp,
        ] {
            let (mut cpu, mut bus) = setup_cpu_and_memory();
            bus.write(TRACE_PC + 1, 0xa6);

            let operations = execute_instruction_with_trace(
                &mut cpu,
                &mut bus,
                &[opcode as u8],
            );

            assert_eq!(
                operations,
                vec![
                    read(TRACE_PC, opcode as u8),
                    read(TRACE_PC + 1, 0xa6),
                ],
                "{opcode:?}"
            );
        }
    }

    #[test]
    fn test_pha_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        bus.write(TRACE_PC + 1, 0xb0);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::PhaImp as u8],
        );

        assert_eq!(cpu.sp, 0xfe);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::PhaImp as u8),
                read(TRACE_PC + 1, 0xb0),
                write(0x01ff, 0x42),
            ]
        );
    }

    #[test]
    fn test_php_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.sr = 0x85;
        bus.write(TRACE_PC + 1, 0xb1);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::PhpImp as u8],
        );

        assert_eq!(cpu.sp, 0xfe);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::PhpImp as u8),
                read(TRACE_PC + 1, 0xb1),
                write(0x01ff, 0xb5),
            ]
        );
    }

    #[test]
    fn test_pla_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.sp = 0xfe;
        bus.write(TRACE_PC + 1, 0xb2);
        bus.write(0x01fe, 0xa0);
        bus.write(0x01ff, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::PlaImp as u8],
        );

        assert_eq!(cpu.a, 0x81);
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::PlaImp as u8),
                read(TRACE_PC + 1, 0xb2),
                read(0x01fe, 0xa0),
                read(0x01ff, 0x81),
            ]
        );
    }

    #[test]
    fn test_plp_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.sp = 0xfe;
        bus.write(TRACE_PC + 1, 0xb3);
        bus.write(0x01fe, 0xa1);
        bus.write(0x01ff, 0xb5);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::PlpImp as u8],
        );

        assert_eq!(cpu.sr, 0xa5);
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::PlpImp as u8),
                read(TRACE_PC + 1, 0xb3),
                read(0x01fe, 0xa1),
                read(0x01ff, 0xb5),
            ]
        );
    }

    #[test]
    fn test_jsr_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x01ff, 0xb4);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::JsrAbs as u8, 0x34, 0x12],
        );

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xfd);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::JsrAbs as u8),
                read(TRACE_PC + 1, 0x34),
                read(0x01ff, 0xb4),
                write(0x01ff, 0x80),
                write(0x01fe, 0x02),
                read(TRACE_PC + 2, 0x12),
            ]
        );
    }

    #[test]
    fn test_rts_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.sp = 0xfd;
        bus.write(TRACE_PC + 1, 0xb5);
        bus.write(0x01fd, 0xa0);
        bus.write(0x01fe, 0x34);
        bus.write(0x01ff, 0x12);
        bus.write(0x1234, 0xc0);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::RtsImp as u8],
        );

        assert_eq!(cpu.pc, 0x1235);
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::RtsImp as u8),
                read(TRACE_PC + 1, 0xb5),
                read(0x01fd, 0xa0),
                read(0x01fe, 0x34),
                read(0x01ff, 0x12),
                read(0x1234, 0xc0),
            ]
        )
    }

    #[test]
    fn test_rti_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.sp = 0xfc;
        bus.write(TRACE_PC + 1, 0xb6);
        bus.write(0x01fc, 0xa0);
        bus.write(0x01fd, 0xb5);
        bus.write(0x01fe, 0x34);
        bus.write(0x01ff, 0x12);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::RtiImp as u8],
        );

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(cpu.sr, 0xa5);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::RtiImp as u8),
                read(TRACE_PC + 1, 0xb6),
                read(0x01fc, 0xa0),
                read(0x01fd, 0xb5),
                read(0x01fe, 0x34),
                read(0x01ff, 0x12),
            ]
        );
    }

    #[test]
    fn test_brk_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.sr = 0x81;
        bus.write(TRACE_PC + 1, 0xb7);
        bus.write(0xfffe, 0x34);
        bus.write(0xffff, 0x12);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::BrkImp as u8],
        );

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xfc);
        assert_eq!(cpu.sr, 0x85);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::BrkImp as u8),
                read(TRACE_PC + 1, 0xb7),
                write(0x01ff, 0x80),
                write(0x01fe, 0x02),
                write(0x01fd, 0xb1),
                read(0xfffe, 0x34),
                read(0xffff, 0x12),
            ]
        );
    }

    #[test]
    fn test_jmp_absolute_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::JmpAbs as u8, 0x34, 0x12],
        );

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::JmpAbs as u8),
                read(TRACE_PC + 1, 0x34),
                read(TRACE_PC + 2, 0x12),
            ]
        );
    }

    #[test]
    fn test_jmp_indirect_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x1234, 0x78);
        bus.write(0x1235, 0x56);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::JmpInd as u8, 0x34, 0x12],
        );

        assert_eq!(cpu.pc, 0x5678);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::JmpInd as u8),
                read(TRACE_PC + 1, 0x34),
                read(TRACE_PC + 2, 0x12),
                read(0x1234, 0x78),
                read(0x1235, 0x56),
            ]
        );
    }

    #[test]
    fn test_jmp_indirect_page_wrap_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x12ff, 0x34);
        bus.write(0x1200, 0x56);
        bus.write(0x1300, 0x78);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::JmpInd as u8, 0xff, 0x12],
        );

        assert_eq!(cpu.pc, 0x5634);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::JmpInd as u8),
                read(TRACE_PC + 1, 0xff),
                read(TRACE_PC + 2, 0x12),
                read(0x12ff, 0x34),
                read(0x1200, 0x56),
            ]
        );
    }

    #[test]
    fn test_cpu_new() {
        let cpu: CPU<crate::test_bus::TestBus> = CPU::new();

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.sr, 0b0000_0000);
        assert_eq!(cpu.sp, 0xff);
        assert_eq!(cpu.pc, 0xff_fc);
        assert_eq!(cpu.cycles, 0);

        assert_eq!(cpu.state, CPUState::FetchOpcode);
        assert_eq!(cpu.opcode, 0x00);
        assert_eq!(cpu.extra_cycle, false);
        assert_eq!(cpu.addr_oper, 0x00_00);
        assert_eq!(cpu.operand, 0x00);
        assert_eq!(cpu.interrupts, 0b0000_0000);
        assert!(!cpu.nmi_line_high);
        assert!(!cpu.irq_accepted);
        assert!(!cpu.irq_poll_i_flag);
        assert!(!cpu.irq_poll_valid);
        assert_eq!(cpu.dma_active, false);
    }

    #[test]
    fn test_fetch_opcode() {
        const PC: u16 = 0x6789;
        const OPCODE: Opcode = Opcode::ClcImp;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(PC, OPCODE as u8);
        cpu.pc = PC;
        cpu.state = CPUState::FetchOpcode;

        cpu.fetch_opcode(&mut mem);

        assert_eq!(cpu.opcode, OPCODE as u8);
        assert_eq!(cpu.addr_mode, AddressingMode::Implied);
        assert_eq!(cpu.state, CPUState::AddressModeResolution);
        assert_eq!(cpu.pc, PC + 1);
        assert_eq!(cpu.cycles, 1);
    }

    #[test]
    fn test_execute_single_instruction() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        let initial_pc = cpu.pc;

        // Set up a NOP instruction
        mem.write(initial_pc, Opcode::NopImp as u8); // NOP opcode

        cpu.execute_single_instruction(&mut mem);

        // Should be back to FetchOpcode state
        assert_eq!(cpu.state, CPUState::FetchOpcode);
        // PC should have advanced
        assert_eq!(cpu.pc, initial_pc + 1);
        // Should have consumed cycles
        assert!(cpu.cycles > 0);
    }

    #[test]
    fn test_set_flag() {
        let (mut cpu, _) = setup_cpu_and_memory();

        // Test setting individual flags
        cpu.set_flag(CARRY_FLAG, true);
        assert_eq!(cpu.sr & CARRY_FLAG, CARRY_FLAG);

        cpu.set_flag(ZERO_FLAG, true);
        assert_eq!(cpu.sr & ZERO_FLAG, ZERO_FLAG);

        cpu.set_flag(NEGATIVE_FLAG, true);
        assert_eq!(cpu.sr & NEGATIVE_FLAG, NEGATIVE_FLAG);

        // Test clearing flags
        cpu.set_flag(CARRY_FLAG, false);
        assert_eq!(cpu.sr & CARRY_FLAG, 0);

        // Test that other flags remain unchanged
        assert_eq!(cpu.sr & ZERO_FLAG, ZERO_FLAG);
        assert_eq!(cpu.sr & NEGATIVE_FLAG, NEGATIVE_FLAG);
    }

    #[test]
    fn test_get_flag() {
        let (mut cpu, _) = setup_cpu_and_memory();

        cpu.sr = CARRY_FLAG | ZERO_FLAG;

        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));
        assert!(!cpu.get_flag(NEGATIVE_FLAG));
        assert!(!cpu.get_flag(OVERFLOW_FLAG));
    }

    #[test]
    fn test_set_multiple_flags() {
        let (mut cpu, _) = setup_cpu_and_memory();

        // Test setting multiple flags at once
        cpu.set_flag(CARRY_FLAG | ZERO_FLAG, true);
        assert!(cpu.get_flag(CARRY_FLAG));
        assert!(cpu.get_flag(ZERO_FLAG));

        // Test clearing multiple flags
        cpu.set_flag(CARRY_FLAG | ZERO_FLAG, false);
        assert!(!cpu.get_flag(CARRY_FLAG));
        assert!(!cpu.get_flag(ZERO_FLAG));
    }

    #[test]
    fn test_fetch_from_and_increment_pc() {
        const PC: u16 = 0x1234;
        const DATA: u8 = 0x42;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(PC, DATA);
        cpu.pc = PC;
        let initial_cycles = cpu.cycles;

        let result = cpu.fetch_from_and_increment_pc(&mut mem);

        assert_eq!(result, DATA);
        assert_eq!(cpu.pc, PC + 1);
        assert_eq!(cpu.cycles, initial_cycles + 1);
    }

    #[test]
    fn test_fetch_from_and_increment_pc_wrapping() {
        const PC: u16 = 0xffff;
        const DATA: u8 = 0x42;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(PC, DATA);
        cpu.pc = PC;

        let result = cpu.fetch_from_and_increment_pc(&mut mem);

        assert_eq!(result, DATA);
        assert_eq!(cpu.pc, 0x0000); // Should wrap around
    }

    #[test]
    fn test_fetch_byte() {
        const ADDR: u16 = 0x5678;
        const DATA: u8 = 0x99;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(ADDR, DATA);
        let initial_cycles = cpu.cycles;

        let result = cpu.fetch_byte(ADDR, &mut mem);

        assert_eq!(result, DATA);
        assert_eq!(cpu.cycles, initial_cycles + 1);
    }

    #[test]
    fn test_write_byte() {
        const ADDR: u16 = 0x9abc;
        const DATA: u8 = 0x77;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        let initial_cycles = cpu.cycles;

        cpu.write_byte(ADDR, DATA, &mut mem);

        assert_eq!(mem.read(ADDR), DATA);
        assert_eq!(cpu.cycles, initial_cycles + 1);
    }

    #[test]
    fn test_fetch_pc_word() {
        const EXPECTED_WORD: u16 = 0x1234;
        const PC: u16 = 0x2000;
        const LOW_BYTE: u8 = 0x34;
        const HIGH_BYTE: u8 = 0x12;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(PC, LOW_BYTE);
        mem.write(PC + 1, HIGH_BYTE);
        cpu.pc = PC;
        let initial_cycles = cpu.cycles;

        let result = cpu.fetch_pc_word(&mut mem);

        assert_eq!(result, EXPECTED_WORD);
        assert_eq!(cpu.pc, PC + 2);
        assert_eq!(cpu.cycles, initial_cycles + 2);
    }

    #[test]
    fn test_full_stack_pointer_address() {
        let (mut cpu, _) = setup_cpu_and_memory();

        cpu.sp = 0xff;
        assert_eq!(cpu.full_stack_pointer_address(), 0x01ff);

        cpu.sp = 0x00;
        assert_eq!(cpu.full_stack_pointer_address(), 0x0100);

        cpu.sp = 0x80;
        assert_eq!(cpu.full_stack_pointer_address(), 0x0180);
    }

    #[test]
    fn test_push_byte_to_stack() {
        const DATA: u8 = 0x42;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        let initial_sp = cpu.sp;
        let initial_cycles = cpu.cycles;
        let stack_addr = cpu.full_stack_pointer_address();

        cpu.push_byte_to_stack(&mut mem, DATA);

        assert_eq!(mem.read(stack_addr), DATA);
        assert_eq!(cpu.sp, initial_sp.wrapping_sub(1));
        assert_eq!(cpu.cycles, initial_cycles + 1);
    }

    #[test]
    fn test_push_word_to_stack() {
        const WORD: u16 = 0x1234;
        const HIGH_BYTE: u8 = 0x12;
        const LOW_BYTE: u8 = 0x34;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        let initial_sp = cpu.sp;
        let initial_cycles = cpu.cycles;

        cpu.push_word_to_stack(&mut mem, WORD);

        // High byte should be pushed first (to higher address)
        assert_eq!(mem.read(0x0100 | initial_sp as u16), HIGH_BYTE);
        assert_eq!(
            mem.read(0x0100 | (initial_sp.wrapping_sub(1)) as u16),
            LOW_BYTE
        );
        assert_eq!(cpu.sp, initial_sp.wrapping_sub(2));
        assert_eq!(cpu.cycles, initial_cycles + 2);
    }

    #[test]
    fn test_pop_byte_from_stack() {
        const DATA: u8 = 0x99;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        // First push something to the stack
        cpu.push_byte_to_stack(&mut mem, DATA);
        let sp_after_push = cpu.sp;
        let cycles_after_push = cpu.cycles;

        // Now pop it back
        let result = cpu.pop_byte_from_stack(&mut mem);

        assert_eq!(result, DATA);
        assert_eq!(cpu.sp, sp_after_push.wrapping_add(1));
        assert_eq!(cpu.cycles, cycles_after_push + 1);
    }

    #[test]
    fn test_stack_underflow_overflow() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        // Test stack pointer wrapping on underflow
        cpu.sp = 0x00;
        cpu.push_byte_to_stack(&mut mem, 0x42);
        assert_eq!(cpu.sp, 0xff);

        // Test stack pointer wrapping on overflow
        cpu.sp = 0xff;
        cpu.pop_byte_from_stack(&mut mem);
        assert_eq!(cpu.sp, 0x00);
    }

    #[test]
    fn test_set_interrupt_flag() {
        let (mut cpu, _) = setup_cpu_and_memory();

        // Test setting NMI pending
        cpu.set_interrupt_flag(NMI_PENDING, true);
        assert_eq!(cpu.interrupts & NMI_PENDING, NMI_PENDING);

        // Test setting IRQ pending
        cpu.set_interrupt_flag(IRQ_PENDING, true);
        assert_eq!(cpu.interrupts & IRQ_PENDING, IRQ_PENDING);

        // Test clearing NMI pending
        cpu.set_interrupt_flag(NMI_PENDING, false);
        assert_eq!(cpu.interrupts & NMI_PENDING, 0);

        // IRQ should still be set
        assert_eq!(cpu.interrupts & IRQ_PENDING, IRQ_PENDING);
    }

    #[test]
    fn test_get_interrupt_flag() {
        let (mut cpu, _) = setup_cpu_and_memory();

        // Initially no interrupts pending
        assert!(!cpu.get_interrupt_flag(NMI_PENDING));
        assert!(!cpu.get_interrupt_flag(IRQ_PENDING));
        assert!(!cpu.get_interrupt_flag(RESET_PENDING));

        // Set some interrupt flags
        cpu.interrupts = NMI_PENDING | RESET_PENDING;

        assert!(cpu.get_interrupt_flag(NMI_PENDING));
        assert!(!cpu.get_interrupt_flag(IRQ_PENDING));
        assert!(cpu.get_interrupt_flag(RESET_PENDING));
    }

    #[test]
    fn test_reset_interrupt() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        // Set up reset vector
        const RESET_VECTOR: u16 = 0x8000;
        mem.write(RESET_VECTOR_LO, (RESET_VECTOR & 0xff) as u8);
        mem.write(RESET_VECTOR_HI, (RESET_VECTOR >> 8) as u8);

        // Set some initial state
        const INITIAL_PC: u16 = 0x1234;
        const INITIAL_SP: u8 = 0x50;
        const INITIAL_SR: u8 = CARRY_FLAG | ZERO_FLAG;

        cpu.pc = INITIAL_PC;
        cpu.sp = INITIAL_SP;
        cpu.sr = INITIAL_SR;
        cpu.set_interrupt_flag(RESET_PENDING, true);

        let initial_cycles = cpu.cycles;
        mem.clear_bus_operations();

        cpu.reset(&mut mem);

        let bus_operations = mem.bus_operations();

        // Check reset behavior
        assert_eq!(cpu.pc, RESET_VECTOR);
        assert_eq!(cpu.sp, INITIAL_SP.wrapping_sub(3));
        assert_eq!(cpu.sr, INITIAL_SR | INTERRUPT_FLAG);
        assert_eq!(cpu.state, CPUState::FetchOpcode);
        assert!(!cpu.get_interrupt_flag(RESET_PENDING));

        assert_eq!(cpu.cycles(), initial_cycles + 7);
        assert_eq!(
            bus_operations,
            vec![
                BusOperation::Read {
                    address: INITIAL_PC,
                    data: 0x00,
                },
                BusOperation::Read {
                    address: INITIAL_PC,
                    data: 0x00,
                },
                BusOperation::Read {
                    address: 0x0150,
                    data: 0x00,
                },
                BusOperation::Read {
                    address: 0x014f,
                    data: 0x00,
                },
                BusOperation::Read {
                    address: 0x014e,
                    data: 0x00,
                },
                BusOperation::Read {
                    address: RESET_VECTOR_LO,
                    data: (RESET_VECTOR & 0xff) as u8,
                },
                BusOperation::Read {
                    address: RESET_VECTOR_HI,
                    data: (RESET_VECTOR >> 8) as u8,
                },
            ]
        );
    }

    #[test]
    fn test_nmi_interrupt() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        // Set up NMI vector
        const NMI_VECTOR: u16 = 0x9000;
        mem.write(NMI_VECTOR_LO, (NMI_VECTOR & 0xff) as u8);
        mem.write(NMI_VECTOR_HI, (NMI_VECTOR >> 8) as u8);

        // Set initial state
        const INITIAL_PC: u16 = 0x1234;
        const INITIAL_SR: u8 = CARRY_FLAG | ZERO_FLAG;
        cpu.pc = INITIAL_PC;
        cpu.sr = INITIAL_SR;
        cpu.sp = 0xff;
        cpu.set_interrupt_flag(NMI_PENDING, true);

        mem.clear_bus_operations();
        let initial_cycles = cpu.cycles;
        cpu.nmi(&mut mem);
        let bus_operations = mem.bus_operations();

        // Check NMI behavior
        assert_eq!(cpu.pc, NMI_VECTOR);
        assert!(cpu.get_flag(INTERRUPT_FLAG));
        assert!(!cpu.get_interrupt_flag(NMI_PENDING));
        assert_eq!(cpu.state, CPUState::FetchOpcode);

        // Check that PC and status were pushed to stack
        assert_eq!(cpu.sp, 0xff - 3); // PC (2 bytes) + status (1 byte)

        // Verify stack contents (PC high byte pushed first)
        assert_eq!(mem.read(0x01ff), (INITIAL_PC >> 8) as u8);
        assert_eq!(mem.read(0x01fe), (INITIAL_PC & 0xff) as u8);
        assert_eq!(mem.read(0x01fd), (INITIAL_SR & !BREAK_FLAG) | UNUSED_FLAG);
        assert_eq!(cpu.cycles, initial_cycles + 7);
        assert_eq!(
            bus_operations,
            vec![
                BusOperation::Read {
                    address: INITIAL_PC,
                    data: 0,
                },
                BusOperation::Read {
                    address: INITIAL_PC,
                    data: 0,
                },
                BusOperation::Write {
                    address: 0x01ff,
                    data: (INITIAL_PC >> 8) as u8,
                },
                BusOperation::Write {
                    address: 0x01fe,
                    data: INITIAL_PC as u8,
                },
                BusOperation::Write {
                    address: 0x01fd,
                    data: (INITIAL_SR & !BREAK_FLAG) | UNUSED_FLAG,
                },
                BusOperation::Read {
                    address: NMI_VECTOR_LO,
                    data: NMI_VECTOR as u8,
                },
                BusOperation::Read {
                    address: NMI_VECTOR_HI,
                    data: (NMI_VECTOR >> 8) as u8,
                },
            ]
        );
    }

    #[test]
    fn test_irq_interrupt() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        // Set up IRQ vector
        const IRQ_VECTOR: u16 = 0xa000;
        mem.write(IRQ_VECTOR_LO, (IRQ_VECTOR & 0xff) as u8);
        mem.write(IRQ_VECTOR_HI, (IRQ_VECTOR >> 8) as u8);

        // Set initial state
        const INITIAL_PC: u16 = 0x5678;
        const INITIAL_SR: u8 = CARRY_FLAG | ZERO_FLAG;
        cpu.pc = INITIAL_PC;
        cpu.sr = INITIAL_SR;
        cpu.sp = 0xff;
        cpu.set_interrupt_flag(IRQ_PENDING, true);

        mem.clear_bus_operations();
        let initial_cycles = cpu.cycles;
        cpu.irq(&mut mem);
        let bus_operations = mem.bus_operations();

        // Check IRQ behavior
        assert_eq!(cpu.pc, IRQ_VECTOR);
        assert!(cpu.get_flag(INTERRUPT_FLAG));
        assert!(!cpu.get_interrupt_flag(IRQ_PENDING));
        assert_eq!(cpu.state, CPUState::FetchOpcode);

        // Check that PC and status were pushed to stack
        assert_eq!(cpu.sp, 0xff - 3);

        // Verify stack contents
        assert_eq!(mem.read(0x01ff), (INITIAL_PC >> 8) as u8);
        assert_eq!(mem.read(0x01fe), (INITIAL_PC & 0xff) as u8);
        assert_eq!(mem.read(0x01fd), (INITIAL_SR & !BREAK_FLAG) | UNUSED_FLAG);
        assert_eq!(cpu.cycles, initial_cycles + 7);
        assert_eq!(
            bus_operations,
            vec![
                BusOperation::Read {
                    address: INITIAL_PC,
                    data: 0,
                },
                BusOperation::Read {
                    address: INITIAL_PC,
                    data: 0,
                },
                BusOperation::Write {
                    address: 0x01ff,
                    data: (INITIAL_PC >> 8) as u8,
                },
                BusOperation::Write {
                    address: 0x01fe,
                    data: INITIAL_PC as u8,
                },
                BusOperation::Write {
                    address: 0x01fd,
                    data: (INITIAL_SR & !BREAK_FLAG) | UNUSED_FLAG,
                },
                BusOperation::Read {
                    address: IRQ_VECTOR_LO,
                    data: IRQ_VECTOR as u8,
                },
                BusOperation::Read {
                    address: IRQ_VECTOR_HI,
                    data: (IRQ_VECTOR >> 8) as u8,
                },
            ]
        );
    }

    #[test]
    fn test_irq_signal_is_serviced_at_instruction_boundary() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        mem.write(0xfffe, IRQ_VECTOR as u8);
        mem.write(0xffff, (IRQ_VECTOR >> 8) as u8);
        mem.set_irq(true);

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, IRQ_VECTOR);
        assert_eq!(cpu.sp, 0xfc);
        assert!(cpu.get_flag(INTERRUPT_FLAG));
        assert_eq!(mem.read(0x01ff), (INITIAL_PC >> 8) as u8);
        assert_eq!(mem.read(0x01fe), INITIAL_PC as u8);
        assert!(cpu.get_interrupt_flag(IRQ_PENDING));

        mem.set_irq(false);
        cpu.sample_interrupts(&mem);
        assert!(!cpu.get_interrupt_flag(IRQ_PENDING));
    }

    #[test]
    fn test_masked_irq_waits_at_instruction_boundary() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        cpu.set_flag(INTERRUPT_FLAG, true);
        mem.write(INITIAL_PC, Opcode::NopImp as u8);
        mem.write(INITIAL_PC + 1, Opcode::NopImp as u8);
        mem.write(0xfffe, IRQ_VECTOR as u8);
        mem.write(0xffff, (IRQ_VECTOR >> 8) as u8);
        mem.set_irq(true);

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, INITIAL_PC + 1);
        assert_eq!(cpu.sp, 0xff);
        assert!(cpu.get_interrupt_flag(IRQ_PENDING));

        cpu.set_flag(INTERRUPT_FLAG, false);
        cpu.execute_single_instruction(&mut mem);

        // Changing the register between calls cannot retroactively change the
        // previous instruction's poll. One instruction establishes a new poll.
        assert_eq!(cpu.pc, INITIAL_PC + 2);

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, IRQ_VECTOR);
        assert_eq!(cpu.sp, 0xfc);
    }

    #[test]
    fn test_cli_delays_pending_irq_by_one_instruction() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        cpu.set_flag(INTERRUPT_FLAG, true);
        mem.write(INITIAL_PC, Opcode::CliImp as u8);
        mem.write(INITIAL_PC + 1, Opcode::NopImp as u8);
        mem.write(IRQ_VECTOR_LO, IRQ_VECTOR as u8);
        mem.write(IRQ_VECTOR_HI, (IRQ_VECTOR >> 8) as u8);
        mem.set_irq(true);

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, INITIAL_PC + 1);
        assert!(!cpu.get_flag(INTERRUPT_FLAG));

        // CLI changes I too late for the poll at the end of CLI. The next
        // instruction must execute before the pending IRQ is acknowledged.
        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, INITIAL_PC + 2);

        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, IRQ_VECTOR);
    }

    #[test]
    fn test_sei_does_not_mask_irq_polled_during_instruction() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        cpu.set_flag(INTERRUPT_FLAG, false);
        mem.write(INITIAL_PC, Opcode::SeiImp as u8);
        mem.write(INITIAL_PC + 1, Opcode::NopImp as u8);
        mem.write(IRQ_VECTOR_LO, IRQ_VECTOR as u8);
        mem.write(IRQ_VECTOR_HI, (IRQ_VECTOR >> 8) as u8);

        // Assert IRQ on SEI's final cycle. Its boundary poll must use the old,
        // clear I value even though SEI leaves I set.
        mem.set_irq_on_cycle(2);
        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, INITIAL_PC + 1);
        assert!(cpu.get_flag(INTERRUPT_FLAG));

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, IRQ_VECTOR);
        assert_ne!(mem.read(0x01fd) & INTERRUPT_FLAG, 0);
    }

    #[test]
    fn test_plp_delays_pending_irq_when_it_clears_i() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        cpu.sp = 0xfe;
        cpu.set_flag(INTERRUPT_FLAG, true);
        mem.write(INITIAL_PC, Opcode::PlpImp as u8);
        mem.write(INITIAL_PC + 1, Opcode::NopImp as u8);
        mem.write(0x01ff, CARRY_FLAG);
        mem.write(IRQ_VECTOR_LO, IRQ_VECTOR as u8);
        mem.write(IRQ_VECTOR_HI, (IRQ_VECTOR >> 8) as u8);
        mem.set_irq(true);

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, INITIAL_PC + 1);
        assert!(!cpu.get_flag(INTERRUPT_FLAG));

        // Like CLI, PLP clears I too late to affect its own boundary poll.
        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, INITIAL_PC + 2);

        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, IRQ_VECTOR);
    }

    #[test]
    fn test_plp_does_not_mask_irq_polled_during_instruction() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        cpu.sp = 0xfe;
        cpu.set_flag(INTERRUPT_FLAG, false);
        mem.write(INITIAL_PC, Opcode::PlpImp as u8);
        mem.write(INITIAL_PC + 1, Opcode::NopImp as u8);
        mem.write(0x01ff, INTERRUPT_FLAG);
        mem.write(IRQ_VECTOR_LO, IRQ_VECTOR as u8);
        mem.write(IRQ_VECTOR_HI, (IRQ_VECTOR >> 8) as u8);

        // Assert IRQ on PLP's final cycle. The poll still sees the old I value.
        mem.set_irq_on_cycle(4);
        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, INITIAL_PC + 1);
        assert!(cpu.get_flag(INTERRUPT_FLAG));

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, IRQ_VECTOR);
        assert_ne!(mem.read(0x01fd) & INTERRUPT_FLAG, 0);
    }

    #[test]
    fn test_rti_immediately_unmasks_irq_when_it_restores_i_clear() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const RETURN_PC: u16 = 0x9000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        cpu.sp = 0xfc;
        cpu.set_flag(INTERRUPT_FLAG, true);
        mem.write(INITIAL_PC, Opcode::RtiImp as u8);
        mem.write(RETURN_PC, Opcode::NopImp as u8);
        mem.write(0x01fd, CARRY_FLAG);
        mem.write(0x01fe, RETURN_PC as u8);
        mem.write(0x01ff, (RETURN_PC >> 8) as u8);
        mem.write(IRQ_VECTOR_LO, IRQ_VECTOR as u8);
        mem.write(IRQ_VECTOR_HI, (IRQ_VECTOR >> 8) as u8);
        mem.set_irq(true);

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, RETURN_PC);
        assert!(!cpu.get_flag(INTERRUPT_FLAG));

        // RTI restores I early enough for the new value to affect its boundary
        // poll, so no instruction at RETURN_PC executes first.
        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, IRQ_VECTOR);
    }

    #[test]
    fn test_rti_immediately_masks_irq_when_it_restores_i_set() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const RETURN_PC: u16 = 0x9000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        cpu.sp = 0xfc;
        cpu.set_flag(INTERRUPT_FLAG, true);
        mem.write(INITIAL_PC, Opcode::RtiImp as u8);
        mem.write(RETURN_PC, Opcode::NopImp as u8);
        mem.write(0x01fd, INTERRUPT_FLAG);
        mem.write(0x01fe, RETURN_PC as u8);
        mem.write(0x01ff, (RETURN_PC >> 8) as u8);
        mem.write(IRQ_VECTOR_LO, IRQ_VECTOR as u8);
        mem.write(IRQ_VECTOR_HI, (IRQ_VECTOR >> 8) as u8);
        mem.set_irq(true);

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, RETURN_PC);
        assert!(cpu.get_flag(INTERRUPT_FLAG));

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, RETURN_PC + 1);
        assert!(cpu.get_interrupt_flag(IRQ_PENDING));
    }

    #[test]
    fn test_brk_does_not_allow_back_to_back_irq_entry() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const IRQ_VECTOR: u16 = 0xa000;

        cpu.pc = INITIAL_PC;
        cpu.set_flag(INTERRUPT_FLAG, false);
        mem.write(INITIAL_PC, Opcode::BrkImp as u8);
        mem.write(INITIAL_PC + 1, 0x00);
        mem.write(IRQ_VECTOR, Opcode::NopImp as u8);
        mem.write(IRQ_VECTOR_LO, IRQ_VECTOR as u8);
        mem.write(IRQ_VECTOR_HI, (IRQ_VECTOR >> 8) as u8);

        // Assert IRQ during BRK. BRK sets I early enough to prevent another
        // interrupt sequence from running before the first handler opcode.
        mem.set_irq_on_cycle(7);
        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, IRQ_VECTOR);
        assert_eq!(cpu.sp, 0xfc);

        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, IRQ_VECTOR + 1);
        assert_eq!(cpu.sp, 0xfc);
    }

    #[test]
    fn test_nmi_is_serviced_only_on_a_rising_edge() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const INITIAL_PC: u16 = 0x8000;
        const NMI_VECTOR: u16 = 0x9000;

        cpu.pc = INITIAL_PC;
        mem.write(0xfffa, NMI_VECTOR as u8);
        mem.write(0xfffb, (NMI_VECTOR >> 8) as u8);
        mem.write(NMI_VECTOR, Opcode::NopImp as u8);
        mem.write(NMI_VECTOR + 1, Opcode::NopImp as u8);
        mem.set_nmi(true);

        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, NMI_VECTOR);
        assert_eq!(cpu.sp, 0xfc);

        // Keeping the NMI line high must not trigger a second interrupt.
        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, NMI_VECTOR + 1);
        assert_eq!(cpu.sp, 0xfc);

        // A new low-to-high transition must trigger another NMI.
        mem.set_nmi(false);
        cpu.execute_single_instruction(&mut mem);
        mem.set_nmi(true);
        cpu.execute_single_instruction(&mut mem);

        assert_eq!(cpu.pc, NMI_VECTOR);
        assert_eq!(cpu.sp, 0xf9);
    }

    #[test]
    fn test_interrupt_priority_is_reset_then_nmi_then_irq() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();
        const RESET_VECTOR: u16 = 0x7000;
        const NMI_VECTOR: u16 = 0x8000;
        const IRQ_VECTOR: u16 = 0x9000;

        mem.write(0xfffc, RESET_VECTOR as u8);
        mem.write(0xfffd, (RESET_VECTOR >> 8) as u8);
        mem.write(0xfffa, NMI_VECTOR as u8);
        mem.write(0xfffb, (NMI_VECTOR >> 8) as u8);
        mem.write(0xfffe, IRQ_VECTOR as u8);
        mem.write(0xffff, (IRQ_VECTOR >> 8) as u8);
        mem.set_reset(true);
        mem.set_nmi(true);
        mem.set_irq(true);

        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, RESET_VECTOR);
        assert!(!cpu.get_interrupt_flag(RESET_PENDING));
        assert!(cpu.get_interrupt_flag(NMI_PENDING));
        assert!(cpu.get_interrupt_flag(IRQ_PENDING));

        mem.set_reset(false);
        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, NMI_VECTOR);
        assert!(!cpu.get_interrupt_flag(NMI_PENDING));
        assert!(cpu.get_interrupt_flag(IRQ_PENDING));

        mem.set_nmi(false);
        cpu.set_flag(INTERRUPT_FLAG, false);
        cpu.execute_single_instruction(&mut mem);
        assert_eq!(cpu.pc, IRQ_VECTOR);
    }

    #[test]
    fn test_fetch_operand() {
        const ADDR: u16 = 0x3000;
        const DATA: u8 = 0x88;

        let (mut cpu, mut mem) = setup_cpu_and_memory();

        mem.write(ADDR, DATA);
        cpu.addr_oper = ADDR;

        cpu.fetch_operand(&mut mem);

        assert_eq!(cpu.operand, DATA);
    }

    #[test]
    fn test_dummy_read() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        const PC: u16 = 0x4000;
        const DATA: u8 = 0x55;

        mem.write(PC, DATA);
        cpu.pc = PC;
        let initial_cycles = cpu.cycles;

        cpu.dummy_read(&mut mem);

        // Should consume a cycle but not change PC or any other state
        assert_eq!(cpu.pc, PC);
        assert_eq!(cpu.cycles, initial_cycles + 1);
    }

    #[test]
    fn test_pc_getter_setter() {
        let (mut cpu, _) = setup_cpu_and_memory();

        const NEW_PC: u16 = 0x1337;

        cpu.set_pc(NEW_PC);
        assert_eq!(cpu.pc(), NEW_PC);
    }

    #[test]
    fn test_cycles_getter() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        let initial_cycles = cpu.cycles();

        // Perform an operation that consumes cycles
        cpu.fetch_byte(0x1000, &mut mem);

        assert_eq!(cpu.cycles(), initial_cycles + 1);
    }

    #[test]
    fn test_set_addr_oper() {
        let (mut cpu, _) = setup_cpu_and_memory();

        const ADDR: u16 = 0xbeef;

        cpu.set_addr_oper(ADDR);
        assert_eq!(cpu.addr_oper, ADDR);
    }

    #[test]
    fn test_sta_zero_page_x_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        cpu.x = 0x20;
        bus.write(0x00f0, 0xa0);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StaZpx as u8, 0xf0],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StaZpx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(0x00f0, 0xa0),
                write(0x0010, 0x42),
            ]
        );
    }

    #[test]
    fn test_stx_zero_page_y_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x5a;
        cpu.y = 0x10;
        bus.write(0x00f8, 0xa1);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StxZpy as u8, 0xf8],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StxZpy as u8),
                read(TRACE_PC + 1, 0xf8),
                read(0x00f8, 0xa1),
                write(0x0008, 0x5a),
            ]
        );
    }

    #[test]
    fn test_sty_zero_page_x_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x02;
        cpu.y = 0x6b;
        bus.write(0x00ff, 0xa2);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StyZpx as u8, 0xff],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StyZpx as u8),
                read(TRACE_PC + 1, 0xff),
                read(0x00ff, 0xa2),
                write(0x0001, 0x6b),
            ]
        );
    }

    #[test]
    fn test_sta_absolute_x_bus_trace_without_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        cpu.x = 0x0f;
        bus.write(0x12ff, 0xa3);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StaAbx as u8, 0xf0, 0x12],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StaAbx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(TRACE_PC + 2, 0x12),
                read(0x12ff, 0xa3),
                write(0x12ff, 0x42),
            ]
        );
    }

    #[test]
    fn test_sta_absolute_x_bus_trace_with_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        cpu.x = 0x20;
        bus.write(0x1210, 0xa4);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StaAbx as u8, 0xf0, 0x12],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StaAbx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(TRACE_PC + 2, 0x12),
                read(0x1210, 0xa4),
                write(0x1310, 0x42),
            ]
        );
    }

    #[test]
    fn test_sta_absolute_y_bus_trace_without_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        cpu.y = 0x0f;
        bus.write(0x200f, 0xa5);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StaAby as u8, 0x00, 0x20],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StaAby as u8),
                read(TRACE_PC + 1, 0x00),
                read(TRACE_PC + 2, 0x20),
                read(0x200f, 0xa5),
                write(0x200f, 0x42),
            ]
        );
    }

    #[test]
    fn test_sta_absolute_y_bus_trace_with_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        cpu.y = 0x20;
        bus.write(0x2010, 0xa6);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StaAby as u8, 0xf0, 0x20],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StaAby as u8),
                read(TRACE_PC + 1, 0xf0),
                read(TRACE_PC + 2, 0x20),
                read(0x2010, 0xa6),
                write(0x2110, 0x42),
            ]
        );
    }

    #[test]
    fn test_sta_indexed_indirect_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        cpu.x = 0x20;
        bus.write(0x00f0, 0xa7);
        bus.write(0x0010, 0x56);
        bus.write(0x0011, 0x34);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StaInx as u8, 0xf0],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StaInx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(0x00f0, 0xa7),
                read(0x0010, 0x56),
                read(0x0011, 0x34),
                write(0x3456, 0x42),
            ]
        );
    }

    #[test]
    fn test_sta_indirect_indexed_bus_trace_without_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        cpu.y = 0x0f;
        bus.write(0x0080, 0xf0);
        bus.write(0x0081, 0x12);
        bus.write(0x12ff, 0xa8);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StaIny as u8, 0x80],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StaIny as u8),
                read(TRACE_PC + 1, 0x80),
                read(0x0080, 0xf0),
                read(0x0081, 0x12),
                read(0x12ff, 0xa8),
                write(0x12ff, 0x42),
            ]
        );
    }

    #[test]
    fn test_sta_indirect_indexed_bus_trace_with_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.a = 0x42;
        cpu.y = 0x20;
        bus.write(0x0080, 0xf0);
        bus.write(0x0081, 0x12);
        bus.write(0x1210, 0xa9);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::StaIny as u8, 0x80],
        );

        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::StaIny as u8),
                read(TRACE_PC + 1, 0x80),
                read(0x0080, 0xf0),
                read(0x0081, 0x12),
                read(0x1210, 0xa9),
                write(0x1310, 0x42),
            ]
        );
    }

    #[test]
    fn test_lda_immediate_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaImm as u8, 0x81],
        );

        assert_eq!(cpu.a, 0x81);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaImm as u8),
                read(TRACE_PC + 1, 0x81),
            ]
        );
    }

    #[test]
    fn test_lda_zero_page_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x0042, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaZpg as u8, 0x42],
        );

        assert_eq!(cpu.a, 0x81);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaZpg as u8),
                read(TRACE_PC + 1, 0x42),
                read(0x0042, 0x81),
            ]
        );
    }

    #[test]
    fn test_lda_zero_page_x_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x20;
        bus.write(0x00f0, 0xa0);
        bus.write(0x0010, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaZpx as u8, 0xf0],
        );

        assert_eq!(cpu.a, 0x81);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaZpx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(0x00f0, 0xa0),
                read(0x0010, 0x81),
            ]
        );
    }

    #[test]
    fn test_lda_absolute_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x1234, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaAbs as u8, 0x34, 0x12],
        );

        assert_eq!(cpu.a, 0x81);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaAbs as u8),
                read(TRACE_PC + 1, 0x34),
                read(TRACE_PC + 2, 0x12),
                read(0x1234, 0x81),
            ]
        );
    }

    #[test]
    fn test_lda_indexed_indirect_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x20;
        bus.write(0x00f0, 0xa1);
        bus.write(0x0010, 0x34);
        bus.write(0x0011, 0x12);
        bus.write(0x1234, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaInx as u8, 0xf0],
        );

        assert_eq!(cpu.a, 0x81);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaInx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(0x00f0, 0xa1),
                read(0x0010, 0x34),
                read(0x0011, 0x12),
                read(0x1234, 0x81),
            ]
        );
    }

    #[test]
    fn test_lda_indexed_indirect_pointer_wrap_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x01;
        bus.write(0x00fe, 0xa2);
        bus.write(0x00ff, 0x34);
        bus.write(0x0000, 0x12);
        bus.write(0x1234, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaInx as u8, 0xfe],
        );

        assert_eq!(cpu.a, 0x81);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaInx as u8),
                read(TRACE_PC + 1, 0xfe),
                read(0x00fe, 0xa2),
                read(0x00ff, 0x34),
                read(0x0000, 0x12),
                read(0x1234, 0x81),
            ]
        );
    }

    #[test]
    fn test_lda_absolute_x_bus_trace_without_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x0f;
        bus.write(0x120f, 0x42);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaAbx as u8, 0x00, 0x12],
        );

        assert_eq!(cpu.a, 0x42);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaAbx as u8),
                read(TRACE_PC + 1, 0x00),
                read(TRACE_PC + 2, 0x12),
                read(0x120f, 0x42),
            ]
        );
    }

    #[test]
    fn test_lda_absolute_x_bus_trace_with_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.x = 0x20;
        bus.write(0x1210, 0xa0);
        bus.write(0x1310, 0x42);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaAbx as u8, 0xf0, 0x12],
        );

        assert_eq!(cpu.a, 0x42);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaAbx as u8),
                read(TRACE_PC + 1, 0xf0),
                read(TRACE_PC + 2, 0x12),
                read(0x1210, 0xa0),
                read(0x1310, 0x42),
            ]
        );
    }

    #[test]
    fn test_ldx_absolute_y_bus_trace_without_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.y = 0x0f;
        bus.write(0x200f, 0x43);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdxAby as u8, 0x00, 0x20],
        );

        assert_eq!(cpu.x, 0x43);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdxAby as u8),
                read(TRACE_PC + 1, 0x00),
                read(TRACE_PC + 2, 0x20),
                read(0x200f, 0x43),
            ]
        );
    }

    #[test]
    fn test_ldx_absolute_y_bus_trace_with_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.y = 0x20;
        bus.write(0x2010, 0xa1);
        bus.write(0x2110, 0x43);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdxAby as u8, 0xf0, 0x20],
        );

        assert_eq!(cpu.x, 0x43);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdxAby as u8),
                read(TRACE_PC + 1, 0xf0),
                read(TRACE_PC + 2, 0x20),
                read(0x2010, 0xa1),
                read(0x2110, 0x43),
            ]
        );
    }

    #[test]
    fn test_lda_indirect_indexed_bus_trace_without_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.y = 0x0f;
        bus.write(0x0080, 0xf0);
        bus.write(0x0081, 0x12);
        bus.write(0x12ff, 0x44);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaIny as u8, 0x80],
        );

        assert_eq!(cpu.a, 0x44);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaIny as u8),
                read(TRACE_PC + 1, 0x80),
                read(0x0080, 0xf0),
                read(0x0081, 0x12),
                read(0x12ff, 0x44),
            ]
        );
    }

    #[test]
    fn test_lda_indirect_indexed_bus_trace_with_page_cross() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.y = 0x20;
        bus.write(0x0080, 0xf0);
        bus.write(0x0081, 0x12);
        bus.write(0x1210, 0xa2);
        bus.write(0x1310, 0x44);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaIny as u8, 0x80],
        );

        assert_eq!(cpu.a, 0x44);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaIny as u8),
                read(TRACE_PC + 1, 0x80),
                read(0x0080, 0xf0),
                read(0x0081, 0x12),
                read(0x1210, 0xa2),
                read(0x1310, 0x44),
            ]
        );
    }

    #[test]
    fn test_lda_indirect_indexed_pointer_wrap_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x00ff, 0x34);
        bus.write(0x0000, 0x12);
        bus.write(0x1234, 0x81);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::LdaIny as u8, 0xff],
        );

        assert_eq!(cpu.a, 0x81);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::LdaIny as u8),
                read(TRACE_PC + 1, 0xff),
                read(0x00ff, 0x34),
                read(0x0000, 0x12),
                read(0x1234, 0x81),
            ]
        );
    }

    #[test]
    fn test_bne_not_taken_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        cpu.set_flag(ZERO_FLAG, true);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::BneRel as u8, 0x05],
        );

        assert_eq!(cpu.pc, TRACE_PC + 2);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::BneRel as u8),
                read(TRACE_PC + 1, 0x05),
            ]
        );
    }

    #[test]
    fn test_bne_taken_same_page_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(TRACE_PC + 2, 0xa0);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::BneRel as u8, 0x05],
        );

        assert_eq!(cpu.pc, TRACE_PC + 7);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::BneRel as u8),
                read(TRACE_PC + 1, 0x05),
                read(TRACE_PC + 2, 0xa0),
            ]
        );
    }

    #[test]
    fn test_bne_taken_backward_page_cross_bus_trace() {
        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(TRACE_PC + 2, 0xa1);
        bus.write(0x80fe, 0xa2);

        let operations = execute_instruction_with_trace(
            &mut cpu,
            &mut bus,
            &[Opcode::BneRel as u8, 0xfc],
        );

        assert_eq!(cpu.pc, 0x7ffe);
        assert_eq!(
            operations,
            vec![
                read(TRACE_PC, Opcode::BneRel as u8),
                read(TRACE_PC + 1, 0xfc),
                read(TRACE_PC + 2, 0xa1),
                read(0x80fe, 0xa2),
            ]
        );
    }

    #[test]
    fn test_bne_taken_forward_page_cross_bus_trace() {
        const INSTRUCTION_PC: u16 = 0x80fd;

        let (mut cpu, mut bus) = setup_cpu_and_memory();
        bus.write(0x80ff, 0xa3);
        bus.write(0x8001, 0xa4);

        let operations = execute_instruction_at_with_trace(
            &mut cpu,
            &mut bus,
            INSTRUCTION_PC,
            &[Opcode::BneRel as u8, 0x02],
        );

        assert_eq!(cpu.pc, 0x8101);
        assert_eq!(
            operations,
            vec![
                read(INSTRUCTION_PC, Opcode::BneRel as u8),
                read(INSTRUCTION_PC + 1, 0x02),
                read(0x80ff, 0xa3),
                read(0x8001, 0xa4),
            ]
        );
    }

    #[test]
    fn test_should_handle_dma() {
        let (mut cpu, mem) = setup_cpu_and_memory();

        // DMA should only be handled on even cycles
        cpu.cycles = 0;
        // Note: TestMemory always returns false for check_dma_request()
        // so this will always be false in our test environment
        assert!(!cpu.should_handle_dma(&mem));

        cpu.cycles = 1;
        assert!(!cpu.should_handle_dma(&mem));

        cpu.cycles = 2;
        assert!(!cpu.should_handle_dma(&mem));
    }

    // Test edge cases and error conditions
    #[test]
    fn test_all_flags_combinations() {
        let (mut cpu, _) = setup_cpu_and_memory();

        let all_flags = [
            CARRY_FLAG,
            ZERO_FLAG,
            INTERRUPT_FLAG,
            DECIMAL_FLAG,
            BREAK_FLAG,
            UNUSED_FLAG,
            OVERFLOW_FLAG,
            NEGATIVE_FLAG,
        ];

        // Test setting each flag individually
        for &flag in &all_flags {
            cpu.sr = 0;
            cpu.set_flag(flag, true);
            assert!(cpu.get_flag(flag), "Failed to set flag: 0x{:02X}", flag);
            assert_eq!(cpu.sr, flag);

            cpu.set_flag(flag, false);
            assert!(!cpu.get_flag(flag), "Failed to clear flag: 0x{:02X}", flag);
            assert_eq!(cpu.sr, 0);
        }

        // Test setting all flags at once
        cpu.sr = 0;
        for &flag in &all_flags {
            cpu.set_flag(flag, true);
        }

        for &flag in &all_flags {
            assert!(
                cpu.get_flag(flag),
                "Flag not set in combination: 0x{:02X}",
                flag
            );
        }
    }

    #[test]
    fn test_memory_boundary_conditions() {
        let (mut cpu, mut mem) = setup_cpu_and_memory();

        // Test reading from address 0x0000
        mem.write(0x0000, 0x11);
        assert_eq!(cpu.fetch_byte(0x0000, &mut mem), 0x11);

        // Test reading from address 0xffff
        mem.write(0xffff, 0x22);
        assert_eq!(cpu.fetch_byte(0xffff, &mut mem), 0x22);

        // Test writing to boundaries
        cpu.write_byte(0x0000, 0x33, &mut mem);
        assert_eq!(mem.read(0x0000), 0x33);

        cpu.write_byte(0xffff, 0x44, &mut mem);
        assert_eq!(mem.read(0xffff), 0x44);
    }
}
