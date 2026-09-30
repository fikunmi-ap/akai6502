//! Simple bus for tests.

use std::cell::RefCell;

use crate::CPU;
use crate::MAX_ADDRESSABLE_MEMORY;
use crate::Bus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusOperation {
    Read { address: u16, data: u8 },
    Write { address: u16, data: u8 },
}

#[derive(Debug)]
pub struct TestBus {
    data: [u8; MAX_ADDRESSABLE_MEMORY],
    irq: bool,
    irq_assert_cycle: Option<u64>,
    nmi: bool,
    reset: bool,
    cycles: u64,
    bus_operations: RefCell<Vec<BusOperation>>,
}

impl TestBus {
    pub fn new() -> Self {
        TestBus {
            data: [0; MAX_ADDRESSABLE_MEMORY],
            irq: false,
            irq_assert_cycle: None,
            nmi: false,
            reset: false,
            cycles: 0,
            bus_operations: RefCell::new(Vec::new()),
        }
    }

    pub fn set_irq(&mut self, irq: bool) {
        self.irq = irq;
    }

    /// Make IRQ active once the bus reaches `cycle`, and keep it active
    /// afterward.
    pub fn set_irq_on_cycle(&mut self, cycle: u64) {
        self.irq_assert_cycle = Some(cycle);
    }

    pub fn set_nmi(&mut self, nmi: bool) {
        self.nmi = nmi;
    }

    pub fn set_reset(&mut self, reset: bool) {
        self.reset = reset;
    }

    pub fn bus_operations(&self) -> Vec<BusOperation> {
        self.bus_operations.borrow().clone()
    }

    pub fn clear_bus_operations(&self) {
        self.bus_operations.borrow_mut().clear();
    }
}

impl Bus for TestBus {
    fn read(&self, addr: u16) -> u8 {
        let data = self.data[addr as usize];
        self.bus_operations.borrow_mut().push(BusOperation::Read {
            address: addr,
            data,
        });
        data
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.bus_operations
            .borrow_mut()
            .push(BusOperation::Write {
                address: addr,
                data,
            });
        self.data[addr as usize] = data;
    }

    fn check_dma_request(&self) -> bool {
        false
    }

    fn check_irq  (&self) -> bool {
        self.irq
            || self
                .irq_assert_cycle
                .is_some_and(|cycle| self.cycles >= cycle)
    }

    fn check_nmi  (&self) -> bool {
        self.nmi
    }

    fn check_reset(&self) -> bool {
        self.reset
    }

    fn cycle(&mut self) {
        self.cycles += 1;
    }

    fn get_active_dma_channel(&self) -> Option<u8> {
        None
    }

    fn perform_dma_transfer(&self, _channel: u8) -> crate::DMATransferStatus {
        crate::DMATransferStatus::Complete
    }
}

pub fn setup_cpu_and_memory() -> (CPU<TestBus>, TestBus) {
    let cpu = CPU::new();
    let mem = TestBus::new();
    (cpu, mem)
}
