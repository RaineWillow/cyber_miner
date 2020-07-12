use crate::api::CodeError;
use serde::Serialize;
use std::fmt;
use std::io::{self, Read};
use std::str::FromStr;

/// a line of assembly
pub enum AssemblyLine {
    Label { name: LabelString },
    MemoryDeclaration { label: LabelString, value: u8 },
    Op(OpCode<LabelString>),
}

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

impl Read for OpCode<u8> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut op = [0u8; 3];
        use OpCode::*;
        let len = match &*self {
            Add => {
                op[0] = 0;
                1
            }
            Sub => {
                op[0] = 1;
                1
            }
            Mul => {
                op[0] = 2;
                1
            }
            Div => {
                op[0] = 3;
                1
            }
            Jmp { label } => {
                op[0] = 4;
                op[1] = *label;
                2
            }
            JmpCondition { label } => {
                op[0] = 5;
                op[1] = *label;
                2
            }
            MovReg { to, from } => {
                op[0] = 6;
                op[1] = to.into();
                op[2] = from.into();
                3
            }
            MovImm { to, from } => {
                op[0] = 7;
                op[1] = to.into();
                op[2] = *from;
                3
            }
            MovAddr { to, from } => {
                op[0] = 8;
                op[1] = to.into();
                op[2] = *from;
                3
            }
            StackGet => {
                op[0] = 9;
                1
            }
            StackSet => {
                op[0] = 10;
                1
            }
            Exec => {
                op[0] = 11;
                1
            }
            Return => {
                op[0] = 12;
                1
            }
            CmpCallAddr { component } => {
                op[0] = 13;
                op[1] = *component;
                2
            }
            CmpCallImm { component } => {
                op[0] = 14;
                op[1] = *component;
                2
            }
            Forward => {
                op[0] = 15;
                1
            }
            Rotate => {
                op[0] = 16;
                1
            }
            Break => {
                op[0] = 17;
                1
            }
            Battery => {
                op[0] = 18;
                1
            }
            InventoryGet => {
                op[0] = 19;
                1
            }
            InventoryDrop => {
                op[0] = 20;
                1
            }
            InventoryItem => {
                op[0] = 21;
                1
            }
            Output => {
                op[0] = 22;
                1
            }
            Noop => {
                op[0] = 23;
                1
            }
        };
        if buf.len() >= op.len() {
            buf.copy_from_slice(&op[..len]);
            Ok(len)
        } else {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                OpcodeReadError,
            ))
        }
    }
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
            Some(token) => token.to_lowercase(),
            None => return Ok(Op(Noop)),
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
    EmptyLine,
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
            Err(err) => errors.push(CodeError {
                line: line_num,
                error: err,
            }),
        }
    }
    if errors.len() == 0 {
        Ok(lines)
    } else {
        Err(errors)
    }
}
