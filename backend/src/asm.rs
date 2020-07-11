use serde::Serialize;
use std::str::FromStr;

/// a line of assembly
pub enum AssemblyLine {
    Label { name: String },
    Op(OpCode<String>),
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
}

impl FromStr for AssemblyLine {
    type Err = AssemblyLineParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.trim().split_whitespace();
        use AssemblyLine::*;
        use AssemblyLineParseError::*;
        use OpCode::*;
        match tokens
            .next()
            .ok_or_else(|| EmptyLine)?
            .to_lowercase()
            .as_str()
        {
            "add" => Ok(Op(Add)),
            "sub" => Ok(Op(Sub)),
            "mul" => Ok(Op(Mul)),
            "div" => Ok(Op(Div)),
            "jmp" => {
                let label = tokens
                    .next()
                    .ok_or_else(|| JmpMissingArgument)?
                    .to_lowercase();
                Ok(Op(Jmp { label }))
            }
            "jmpc" => {
                let label = tokens
                    .next()
                    .ok_or_else(|| JmpcMissingArgument)?
                    .to_lowercase();
                Ok(Op(JmpCondition { label }))
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
                        Err(_) => Ok(Op(MovAddr {
                            to,
                            from: from_token.into(),
                        })),
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
                    Err(_) => Ok(Op(CmpCallAddr {
                        component: component.into(),
                    })),
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
                    // Try to parse as register
                    match Register::from_str(line) {
                        Ok(_) => Err(InvalidLabelNameRegister),
                        Err(_) => {
                            if line.chars().all(|c| c.is_ascii_digit()) {
                                Err(InvalidLabelNameNumber)
                            } else {
                                Ok(Label { name: line.into() })
                            }
                        }
                    }
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
    JmpMissingArgument,
    JmpcMissingArgument,
    MovMissingTo,
    MovInvalidToRegister(<Register as FromStr>::Err),
    MovMissingFrom,
    CmpCallMissingComponent,
    InvalidLabelNameRegister,
    InvalidLabelNameNumber,
    InvalidInstruction,
}

pub enum Register {
    RGA,
    RGB,
    RGC,
    RGD,
    RET,
    MEM,
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
            "rga" => Ok(RGA),
            "rgb" => Ok(RGB),
            "rgc" => Ok(RGC),
            "rgd" => Ok(RGD),
            "ret" => Ok(RET),
            "mem" => Ok(MEM),
            _ => Err(InvalidRegister),
        }
    }
}
