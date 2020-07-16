use crate::api::CodeError;
use crate::robot::{BIOS_MEM_SIZE, DATA_MEM_SIZE};
use log::debug;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::io::{self, Read};
use std::str::FromStr;

/// a line of assembly
pub enum AssemblyLine {
    EmptyLine,
    Label { name: LabelString },
    MemoryDeclaration { label: LabelString, value: u8 },
    Op(OpCode<LabelString>),
}
const OP_ADD: u8 = 0;
const OP_SUB: u8 = 1;
const OP_MUL: u8 = 2;
const OP_DIV: u8 = 3;
const OP_JMP: u8 = 4;
const OP_JMPC: u8 = 5;
const OP_MOVR: u8 = 6;
const OP_MOVIMM: u8 = 7;
const OP_MOVADDR: u8 = 8;
const OP_STACKGET: u8 = 9;
const OP_STACKSET: u8 = 10;
const OP_EXEC: u8 = 11;
const OP_RETURN: u8 = 12;
const OP_CMPCALLADDR: u8 = 13;
const OP_CMPCALLIMM: u8 = 14;
const OP_FORWARD: u8 = 15;
const OP_ROTATE: u8 = 16;
const OP_BREAK: u8 = 17;
const OP_BATTERY: u8 = 18;
const OP_INVENTORYGET: u8 = 19;
const OP_INVENTORYDROP: u8 = 20;
const OP_INVENTORYITEM: u8 = 21;
const OP_OUTPUT: u8 = 22;
const OP_NOOP: u8 = 23;

pub enum OpCode<L> {
    Add,
    Sub,
    Mul,
    Div,
    Jmp { label: L },
    JmpCondition { label: L },
    MovReg { to: Register, from: Register },
    MovImm { to: Register, from: u8 },
    MovAddr { to: Register, from: L },
    StackGet,
    StackSet,
    Exec,
    Return,
    CmpCallAddr { component: L },
    CmpCallImm { component: u8 },
    Forward,
    Rotate,
    Break,
    Battery,
    InventoryGet,
    InventoryDrop,
    InventoryItem,
    Output,
    Noop,
}

impl OpCode<LabelString> {
    fn placeholder_labels(&self) -> (OpCode<u8>, Option<(bool, u8, String)>) {
        use OpCode::*;
        match *self {
            Add => (Add, None),
            Sub => (Sub, None),
            Mul => (Mul, None),
            Div => (Div, None),
            Jmp { ref label } => (Jmp { label: 0 }, Some((true, 1, label.0.clone()))),
            JmpCondition { ref label } => {
                (JmpCondition { label: 0 }, Some((true, 1, label.0.clone())))
            }
            MovReg { to, from } => (MovReg { to, from }, None),
            MovImm { to, from } => (MovImm { to, from }, None),
            MovAddr { to, ref from } => (
                MovAddr { to: to, from: 0 },
                Some((false, 2, from.0.clone())),
            ),
            StackGet => (StackGet, None),
            StackSet => (StackSet, None),
            Exec => (Exec, None),
            Return => (Return, None),
            CmpCallAddr { ref component } => (
                CmpCallAddr { component: 0 },
                Some((false, 1, component.0.clone())),
            ),
            CmpCallImm { component } => (CmpCallImm { component }, None),

            Forward => (Forward, None),
            Rotate => (Rotate, None),
            Break => (Break, None),
            Battery => (Battery, None),
            InventoryGet => (InventoryGet, None),
            InventoryDrop => (InventoryDrop, None),
            InventoryItem => (InventoryItem, None),
            Output => (Output, None),
            Noop => (Noop, None),
            _ => unimplemented!(),
        }
    }
}
impl Read for OpCode<u8> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut op = [0u8; 3];
        use OpCode::*;
        let len = match &*self {
            Add => {
                op[0] = OP_ADD;
                1
            }
            Sub => {
                op[0] = OP_SUB;
                1
            }
            Mul => {
                op[0] = OP_MUL;
                1
            }
            Div => {
                op[0] = OP_DIV;
                1
            }
            Jmp { label } => {
                op[0] = OP_JMP;
                op[1] = *label;
                2
            }
            JmpCondition { label } => {
                op[0] = OP_JMPC;
                op[1] = *label;
                2
            }
            MovReg { to, from } => {
                op[0] = OP_MOVR;
                op[1] = to.into();
                op[2] = from.into();
                3
            }
            MovImm { to, from } => {
                op[0] = OP_MOVIMM;
                op[1] = to.into();
                op[2] = *from;
                3
            }
            MovAddr { to, from } => {
                op[0] = OP_MOVADDR;
                op[1] = to.into();
                op[2] = *from;
                3
            }
            StackGet => {
                op[0] = OP_STACKGET;
                1
            }
            StackSet => {
                op[0] = OP_STACKSET;
                1
            }
            Exec => {
                op[0] = OP_EXEC;
                1
            }
            Return => {
                op[0] = OP_RETURN;
                1
            }
            CmpCallAddr { component } => {
                op[0] = OP_CMPCALLADDR;
                op[1] = *component;
                2
            }
            CmpCallImm { component } => {
                op[0] = OP_CMPCALLIMM;
                op[1] = *component;
                2
            }
            Forward => {
                op[0] = OP_FORWARD;
                1
            }
            Rotate => {
                op[0] = OP_ROTATE;
                1
            }
            Break => {
                op[0] = OP_BREAK;
                1
            }
            Battery => {
                op[0] = OP_BATTERY;
                1
            }
            InventoryGet => {
                op[0] = OP_INVENTORYGET;
                1
            }
            InventoryDrop => {
                op[0] = OP_INVENTORYDROP;
                1
            }
            InventoryItem => {
                op[0] = OP_INVENTORYITEM;
                1
            }
            Output => {
                op[0] = OP_OUTPUT;
                1
            }
            Noop => {
                op[0] = OP_NOOP;
                1
            }
        };
        if buf.len() >= op.len() {
            buf[..len].copy_from_slice(&op[..len]);
            Ok(len)
        } else {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                OpcodeReadError,
            ))
        }
    }
}

impl OpCode<u8> {
    pub fn read_from(memory: &[u8]) -> Result<(Self, u8), ReadOpCodeError> {
        // Get a memory address
        if memory.len() == 0 {
            return Err(ReadOpCodeError::OutOfBounds);
        }
        match memory[0] {
            OP_ADD => Ok((OpCode::Add, 1)),
            OP_SUB => Ok((OpCode::Sub, 1)),
            OP_MUL => Ok((OpCode::Mul, 1)),
            OP_DIV => Ok((OpCode::Div, 1)),
            OP_JMP => Ok((OpCode::Jmp { label: 0 }, 2)),
            OP_JMPC => Ok((OpCode::JmpCondition { label: 0 }, 1)),
            OP_MOVR => Ok((
                OpCode::MovReg {
                    to: Register::RGA,
                    from: Register::RGB,
                },
                1,
            )),
            OP_MOVIMM => Ok((
                OpCode::MovImm {
                    to: Register::RGA,
                    from: 0,
                },
                1,
            )),
            OP_MOVADDR => Ok((
                OpCode::MovAddr {
                    to: Register::RGA,
                    from: 0,
                },
                1,
            )),
            OP_STACKGET => Ok((OpCode::StackGet, 1)),
            OP_STACKSET => Ok((OpCode::StackSet, 1)),
            OP_EXEC => Ok((OpCode::Exec, 1)),
            OP_RETURN => Ok((OpCode::Return, 1)),
            OP_CMPCALLADDR => Ok((OpCode::CmpCallAddr { component: 0 }, 1)),
            OP_CMPCALLIMM => Ok((OpCode::CmpCallImm { component: 0 }, 1)),
            OP_FORWARD => Ok((OpCode::Forward, 1)),
            OP_ROTATE => Ok((OpCode::Rotate, 1)),
            OP_BREAK => Ok((OpCode::Break, 1)),
            OP_BATTERY => Ok((OpCode::Battery, 1)),
            OP_INVENTORYGET => Ok((OpCode::InventoryGet, 1)),
            OP_INVENTORYDROP => Ok((OpCode::InventoryDrop, 1)),
            OP_INVENTORYITEM => Ok((OpCode::InventoryItem, 1)),
            OP_OUTPUT => Ok((OpCode::Output, 1)),
            OP_NOOP => Ok((OpCode::Noop, 1)),
            _ => {
                return Err(ReadOpCodeError::InvalidOpcode);
            }
        }
    }
}
pub enum ReadOpCodeError {
    OutOfBounds,
    InvalidOpcode,
}

#[derive(Debug)]
struct OpcodeReadError;
impl fmt::Display for OpcodeReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Buffer too small")
    }
}
impl std::error::Error for OpcodeReadError {}

impl FromStr for AssemblyLine {
    type Err = AssemblyLineParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.trim().split_whitespace();
        use AssemblyLine::*;
        use AssemblyLineParseError::*;
        use OpCode::*;
        let first_token = match tokens.next() {
            Some(token) => {
                if token.starts_with("//") {
                    return Ok(EmptyLine);
                } else {
                    token.to_lowercase()
                }
            }
            None => return Ok(EmptyLine),
        };
        match first_token.as_str() {
            "let" => {
                let label = tokens
                    .next()
                    .ok_or_else(|| LetMissingLabel)?
                    .to_lowercase()
                    .parse()
                    .map_err(LetInvalidLabel)?;
                tokens
                    .next()
                    .ok_or_else(|| LetMissingEquals)
                    .and_then(|symbol| {
                        if symbol == "=" {
                            Ok(())
                        } else {
                            Err(LetMissingEquals)
                        }
                    })?;
                let value = tokens
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .map_err(|_| LetInvalidValue)?;
                Ok(MemoryDeclaration { label, value })
            }
            "add" => Ok(Op(Add)),
            "sub" => Ok(Op(Sub)),
            "mul" => Ok(Op(Mul)),
            "div" => Ok(Op(Div)),
            "jmp" => {
                let label = tokens
                    .next()
                    .ok_or_else(|| JmpMissingArgument)?
                    .to_lowercase();
                LabelString::from_str(&label)
                    .map(|label| Op(Jmp { label }))
                    .map_err(JmpInvalidArgument)
            }
            "jmpc" => {
                let label = tokens
                    .next()
                    .ok_or_else(|| JmpcMissingArgument)?
                    .to_lowercase();
                LabelString::from_str(&label)
                    .map(|label| Op(JmpCondition { label }))
                    .map_err(JmpcInvalidArgument)
            }
            "mov" => {
                let to: Register = tokens
                    .next()
                    .ok_or_else(|| MovMissingTo)?
                    .parse()
                    .map_err(MovInvalidToRegister)?;
                let from_token = tokens.next().ok_or_else(|| MovMissingFrom)?;
                match Register::from_str(from_token) {
                    Ok(register) => Ok(Op(MovReg { to, from: register })),
                    Err(_) => match u8::from_str(from_token) {
                        Ok(imm) => Ok(Op(MovImm { to, from: imm })),
                        Err(_) => LabelString::from_str(from_token)
                            .map(|from_label| {
                                Op(MovAddr {
                                    to,
                                    from: from_label,
                                })
                            })
                            .map_err(MovInvalidFrom),
                    },
                }
            }
            "stac_g" => Ok(Op(StackGet)),
            "stac_s" => Ok(Op(StackSet)),
            "exec" => Ok(Op(Exec)),
            "return" => Ok(Op(Return)),
            "cmp_call" => {
                let component = tokens.next().ok_or_else(|| CmpCallMissingComponent)?;
                match u8::from_str(component) {
                    Ok(imm) => Ok(Op(CmpCallImm { component: imm })),
                    Err(_) => LabelString::from_str(component)
                        .map(|component_label| {
                            Op(CmpCallAddr {
                                component: component_label,
                            })
                        })
                        .map_err(MovInvalidFrom),
                }
            }
            "fwd" => Ok(Op(Forward)),
            "rot" => Ok(Op(Rotate)),
            "brk" => Ok(Op(Break)),
            "bttry" => Ok(Op(Battery)),
            "inven" => Ok(Op(InventoryGet)),
            "drop" => Ok(Op(InventoryDrop)),
            "item" => Ok(Op(InventoryItem)),
            "out" => Ok(Op(Output)),
            "noop" => Ok(Op(Noop)),
            line => {
                if line.ends_with(':') {
                    let line = line.trim_end_matches(':');
                    // Add a dollar sign to parse it like a label
                    let line = format!("${}", line);
                    let label = LabelString::from_str(&line).map_err(InvalidLabel)?;
                    Ok(Label { name: label })
                } else {
                    Err(InvalidInstruction)
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum AssemblyLineParseError {
    LetMissingLabel,
    LetInvalidLabel(LabelParseError),
    LetMissingEquals,
    LetInvalidValue,
    JmpMissingArgument,
    JmpInvalidArgument(LabelParseError),
    JmpcMissingArgument,
    JmpcInvalidArgument(LabelParseError),
    MovMissingTo,
    MovInvalidToRegister(<Register as FromStr>::Err),
    MovMissingFrom,
    MovInvalidFrom(LabelParseError),
    CmpCallMissingComponent,
    InvalidLabel(LabelParseError),
    InvalidInstruction,
}

pub struct LabelString(String);

impl FromStr for LabelString {
    type Err = LabelParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use LabelParseError::*;
        // Check if the string starts with our prefix
        if s.starts_with('$') || s.starts_with('@') {
            // Remove the dollar sign and parse as a label
            let label = s
                .trim_start_matches('$')
                .trim_start_matches('@')
                .to_lowercase();
            // Try to parse it as a register
            if u8::from_str(&label).is_ok() {
                Err(NumericName)
            } else {
                let maybe_register_label = format!("%{}", label);
                if Register::from_str(&maybe_register_label).is_ok() {
                    Err(RegisterName)
                } else {
                    Ok(Self(label.into()))
                }
            }
        } else {
            Err(InvalidPrefix)
        }
    }
}

#[derive(Debug, Serialize)]
pub enum LabelParseError {
    InvalidPrefix,
    NumericName,
    RegisterName,
}

#[derive(Clone, Copy)]
pub enum Register {
    RGA,
    RGB,
    RGC,
    RGD,
    RET,
    MEM,
}

impl Into<u8> for &Register {
    fn into(self) -> u8 {
        use Register::*;
        match self {
            RGA => 0,
            RGB => 1,
            RGC => 2,
            RGD => 3,
            RET => 4,
            MEM => 5,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum RegisterParseError {
    InvalidRegister,
}

impl FromStr for Register {
    type Err = RegisterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Register::*;
        use RegisterParseError::*;
        match s.to_lowercase().as_str() {
            "%rga" => Ok(RGA),
            "%rgb" => Ok(RGB),
            "%rgc" => Ok(RGC),
            "%rgd" => Ok(RGD),
            "%ret" => Ok(RET),
            "%mem" => Ok(MEM),
            _ => Err(InvalidRegister),
        }
    }
}

pub fn parse_code(code: String) -> Result<Vec<AssemblyLine>, Vec<CodeError>> {
    // Stores valid code lines
    let mut lines = Vec::new();
    // Stores errors
    let mut errors = Vec::new();
    // Iterate over the given lines
    for (line_num, line) in code.lines().enumerate() {
        match AssemblyLine::from_str(&line) {
            Ok(line) => lines.push(line),
            Err(err) => {
                println!("line: {}", line);
                println!("error: {:?}", err);
                errors.push(CodeError {
                    line: line_num,
                    error: err,
                })
            }
        }
    }
    if errors.len() == 0 {
        Ok(lines)
    } else {
        Err(errors)
    }
}

pub fn assemble(
    code: &[AssemblyLine],
    bios_memory: &mut [u8; BIOS_MEM_SIZE],
    data_memory: &mut [u8; DATA_MEM_SIZE],
) -> Result<(), AssemblingError> {
    // Mapping of program label names to addresses
    let mut label_locations = HashMap::new();
    // Mapping of memory label names to addresses
    let mut memory_locations = HashMap::new();
    // Mapping of program label names to addresses where the should be an argument but the value is
    // not known
    let mut need_labels: HashMap<String, Vec<u8>> = HashMap::new();
    // Mapping of memory label names to addresses where the should be an argument but the value is
    // not known
    let mut need_memory_labels: HashMap<String, Vec<u8>> = HashMap::new();
    // Current offset in the program memory
    let mut cur_offset: u8 = 0;
    // Current offset in data memory
    let mut cur_data_offset: u8 = 0;

    use AssemblingError::*;
    for (cur_line_num, line) in code.iter().enumerate() {
        use AssemblyLine::*;
        match line {
            EmptyLine => {}
            Label { name } => {
                if let Some((defined_line, _)) = label_locations.get(&name.0) {
                    return Err(DuplicateLabel {
                        name: name.0.clone(),
                        first_defined: *defined_line,
                        redefined: cur_line_num,
                    });
                }
                label_locations.insert(name.0.clone(), (cur_line_num, cur_offset));
            }
            MemoryDeclaration { label, value } => {
                if let Some((defined_line, _, _)) = memory_locations.get(&label.0) {
                    return Err(DuplicateMemory {
                        name: label.0.clone(),
                        first_defined: *defined_line,
                        redefined: cur_line_num,
                    });
                }
                memory_locations.insert(label.0.clone(), (cur_line_num, cur_data_offset, value));
                cur_data_offset += 1;
            }
            Op(opcode) => {
                // Get a placeholder for the offset
                let (mut placeholder, label_info) = opcode.placeholder_labels();
                // Write the placeholder to memory
                let bytes_written = placeholder
                    .read(&mut bios_memory[usize::from(cur_offset)..])
                    .map_err(MemoryOverflow)?;
                // Add label placeholder info if needed
                if let Some((is_program_label, arg_offset, label_name)) = label_info {
                    let label_offset = cur_offset.checked_add(arg_offset).unwrap();
                    if is_program_label {
                        if let Some((_, label_address)) = label_locations.get(&label_name) {
                            bios_memory[usize::from(label_offset)] = *label_address;
                        } else {
                            need_labels
                                .entry(label_name)
                                .and_modify(|v| v.push(label_offset))
                                .or_insert_with(|| vec![label_offset]);
                        }
                    } else {
                        if let Some((_, memory_address, _)) = memory_locations.get(&label_name) {
                            bios_memory[usize::from(label_offset)] = *memory_address;
                        } else {
                            need_memory_labels
                                .entry(label_name)
                                .and_modify(|v| v.push(label_offset))
                                .or_insert_with(|| vec![]);
                        }
                    }
                }
                // Move the offset forward
                let bytes_written: u8 = bytes_written.try_into().map_err(|_| PointerOverflow)?;
                cur_offset = cur_offset
                    .checked_add(bytes_written)
                    .ok_or_else(|| PointerOverflow)?;
            }
        }
    }
    // Iterate over all the locations that need program memory labels filled in
    for (label_name, offsets) in &need_labels {
        for offset in offsets {
            if let Some((_, label_address)) = label_locations.get(label_name) {
                bios_memory[usize::from(*offset)] = *label_address;
                debug!(
                    "Writing address {} for label {} to {}",
                    label_name, label_address, offset
                );
            } else {
                return Err(InvalidLabel);
            }
        }
    }
    // Iterate over all the locations that need program memory labels filled in
    for (label_name, offsets) in &need_memory_labels {
        for offset in offsets {
            if let Some((_, label_address, _)) = memory_locations.get(label_name) {
                data_memory[usize::from(*offset)] = *label_address;
                debug!(
                    "Writing address {} for label {} to {}",
                    label_name, label_address, offset
                );
            } else {
                return Err(InvalidMemoryLabel);
            }
        }
    }
    // Wipe data memory
    for d in data_memory.iter_mut() {
        *d = 0;
    }
    // Fill in data memory with defined values
    for (_, (_, address, value)) in memory_locations {
        data_memory[usize::from(address)] = *value;
    }
    dbg!(&need_labels);
    dbg!(&need_memory_labels);
    Ok(())
}

#[derive(Debug)]
pub enum AssemblingError {
    DuplicateLabel {
        name: String,
        first_defined: usize,
        redefined: usize,
    },
    DuplicateMemory {
        name: String,
        first_defined: usize,
        redefined: usize,
    },
    MemoryOverflow(io::Error),
    PointerOverflow,
    InvalidLabel,
    InvalidMemoryLabel,
}
