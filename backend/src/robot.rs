const STEPS_PER_CHARGE_LEVEL: u8 = 5;
const BIOS_MEM_SIZE: usize = 256;
const DATA_MEM_SIZE: usize = 256;
const PROG_MEM_SIZE: usize = 256;
const CALL_STACK_LEN: usize = 16;

pub struct Robot {
    /// Input Registers
    reg: Registers,
    /// Output register
    ret: u8,
    /// BIOS memory
    bios: [u8; BIOS_MEM_SIZE],
    /// BIOS call stack
    bios_call_stack: [u8; CALL_STACK_LEN],
    /// Current position in the BIOS call stack
    bios_call_stack_pos: u8,
    /// General-use memory
    memory: [u8; DATA_MEM_SIZE],
    /// Program memory
    progmem: [u8; PROG_MEM_SIZE],
    /// Program call stack
    prog_call_stack: [u8; CALL_STACK_LEN],
    /// Current position in the Program call stack
    prog_call_stack_pos: u8,
    /// Components
    components: Vec<Box<Component>>,
    /// Inventory
    inventory: Vec<Item>,
    /// Battery charge
    ///
    /// Gets divided by STEPS_PER_CHARGE_LEVEL before returning
    battery: u16,
    /// Current instruction being executed
    sp: usize,
}

/// Input register
pub struct Registers {
    rga: u8,
    rgb: u8,
    rgc: u8,
    rgd: u8,
    mem: u8,
}

trait Component {
    /// When a component is called, this is run
    ///
    /// # Parameters
    /// * `reg` -
    fn call(
        &mut self,
        reg: Registers,
        memory: &mut [u8; DATA_MEM_SIZE],
        prog_mem: &mut [u8; PROG_MEM_SIZE],
    ) -> u8;
}

struct Item {
    id: u8,
}