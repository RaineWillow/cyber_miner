const STEPS_PER_CHARGE_LEVEL: u8 = 5;
const BIOS_MEM_SIZE: usize = 256;
const DATA_MEM_SIZE: usize = 256;
const PROG_MEM_SIZE: usize = 256;
const CALL_STACK_LEN: usize = 16;

#[derive(Clone)]
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
    // TODO: put a component here
    components: Vec<()>,
    /// Inventory
    inventory: Vec<Item>,
    /// Battery charge
    ///
    /// Gets divided by STEPS_PER_CHARGE_LEVEL before returning
    battery: u16,
    /// Current instruction being executed in the bios
    sp: usize,
    /// Current instruction being executed in progmem
    psp: usize,
    /// X position
    x: usize,
    /// Y position
    y: usize,
}

impl Default for Robot {
    fn default() -> Self {
        Self {
            reg: Registers::default(),
            ret: 0,
            bios: [0; BIOS_MEM_SIZE],
            bios_call_stack: [0; CALL_STACK_LEN],
            bios_call_stack_pos: 0,
            memory: [0; DATA_MEM_SIZE],
            progmem: [0; PROG_MEM_SIZE],
            prog_call_stack: [0; CALL_STACK_LEN],
            prog_call_stack_pos: 0,
            components: vec![],
            inventory: vec![],
            battery: u16::from(STEPS_PER_CHARGE_LEVEL) * u16::from(u8::MAX),
            sp: 0,
            psp: 0,
            x: 0,
            y: 0,
        }
    }
}

/// Input registers
#[derive(Clone, Copy, Default)]
pub struct Registers {
    rga: u8,
    rgb: u8,
    rgc: u8,
    rgd: u8,
    mem: u8,
}

#[derive(Clone, Copy)]
struct Item {
    id: u8,
}
