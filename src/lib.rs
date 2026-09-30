//! High-performance, cycle-accurate, no-dependency emulation of the 6502.


mod instructions;
mod addressing;
mod cpu;

#[cfg(test)]
mod single_step_tests;

pub mod opcodes;
pub mod test_bus;

pub use cpu::CPU;

/// Type alias for a Byte.
pub(crate) type Byte = u8;

/// Type alias for a 6502 Word, which is 16 bits.
pub(crate) type Word = u16;

/// 2 ** 16 units of memory
pub const MAX_ADDRESSABLE_MEMORY: usize = 1024 * 64;

// CPU Flags //
pub const CARRY_FLAG    : u8 = 0b0000_0001;
pub const ZERO_FLAG     : u8 = 0b0000_0010;
pub const INTERRUPT_FLAG: u8 = 0b0000_0100;
pub const DECIMAL_FLAG  : u8 = 0b0000_1000;
pub const BREAK_FLAG    : u8 = 0b0001_0000;
// Note: Does not exist in hardware; only added for simplicity.
pub const UNUSED_FLAG   : u8 = 0b0010_0000;
pub const OVERFLOW_FLAG : u8 = 0b0100_0000;
pub const NEGATIVE_FLAG : u8 = 0b1000_0000;

// Self-defined interrupt flags
pub const NMI_PENDING  : u8 = 0b0000_0001;
pub const IRQ_PENDING  : u8 = 0b0000_0010;
pub const RESET_PENDING: u8 = 0b0000_0100;

// Interrupt vector addresses
pub const NMI_VECTOR_LO  : Word = 0xfffa;
pub const NMI_VECTOR_HI  : Word = 0xfffb;
pub const RESET_VECTOR_LO: Word = 0xfffc;
pub const RESET_VECTOR_HI: Word = 0xfffd;
pub const IRQ_VECTOR_LO  : Word = 0xfffe;
pub const IRQ_VECTOR_HI  : Word = 0xffff;

/// The Bus is how the 6502 interacts with the rest of the world. It is a
/// communication hub and timing coordinator.
/// 
/// In a real 6502, the bus connects all components and handles synchronization
/// between them. This trait abstracts that functionality.
/// 
/// Key Responsibilities
/// - Memory Access
/// - Interrupt Handling
/// - DMA Transfers
/// - Timing.
/// 
/// After each CPU cycle, the bus is given control to:
/// 1. Run other components (PPU, APU, etc) at their relative speeds
/// 2. Handle inter-component communication
/// 3. Maintain system-wide timing accuracy
/// 
/// Example usage for NES:
/// ```ignore
/// impl Bus for NESBus {
///     fn cycle(&mut self) {
///         // PPU runs at 3x CPU speed
///         for _ in 0..3 {
///             self.ppu.step();
///         }
///         
///         // APU runs at CPU speed
///         self.apu.step();
///     }
/// }
/// ```
pub trait Bus {
    fn read(&self, address: Word) -> Byte;
    fn write(&mut self, address: Word, data: Byte);

    /// Advance all components by 1 CPU cycle worth of time.
    fn cycle(&mut self) {}

    // To deal with hardware interrupts.
    fn check_nmi  (&self) -> bool;
    fn check_irq  (&self) -> bool;
    fn check_reset(&self) -> bool;

    // To deal with DMA.
    fn check_dma_request(&self) -> bool;
    fn get_active_dma_channel(&self) -> Option<u8>;
    fn perform_dma_transfer(&self, channel: u8) -> DMATransferStatus;
}

pub enum DMATransferStatus {
    Complete,
    InProgress {bytes_remaining: u8},
    Aborted,
}
