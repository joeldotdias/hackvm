use std::str::FromStr;

static mut NEXTJUMP: u32 = 0;

#[derive(Debug)]
pub enum MemorySegment {
    Local,
    Argument,
    This,
    That,
    Constant,
    Static,
    Temp,
    Pointer,
}

#[derive(Debug)]
pub enum Command {
    Push { segment: MemorySegment, offset: u16 },
    Pop { segment: MemorySegment, offset: u16 },

    Add,
    Sub,
    Neg,

    Not,
    Or,
    And,
    Eq,
    Lt,
    Gt,

    Label,
    Goto,
    If,
    Function,
    Return,
    Call,
}

pub fn parse(line: &str) -> Result<Command, String> {
    let parts: Vec<_> = line.split_whitespace().collect();
    let command = match parts[0] {
        "push" => Command::Push {
            segment: MemorySegment::from_str(parts[1])?,
            offset: parts[2].parse::<u16>().map_err(|e| e.to_string())?,
        },

        "pop" => Command::Pop {
            segment: MemorySegment::from_str(parts[1])?,
            offset: parts[2].parse::<u16>().map_err(|e| e.to_string())?,
        },

        "add" => Command::Add,
        "sub" => Command::Sub,
        "neg" => Command::Neg,
        "not" => Command::Not,
        "or" => Command::Or,
        "and" => Command::And,
        "eq" => Command::Eq,
        "lt" => Command::Lt,
        "gt" => Command::Gt,

        _ => return Err(format!("Unknown command {}", parts[0])),
    };

    Ok(command)
}

impl Command {
    pub fn to_asm(&self, filename: &str) -> String {
        self.verify_offset();

        match self {
            Command::Push { segment, offset } => match segment {
                MemorySegment::Constant => format!("@{}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", offset),
                MemorySegment::Static => {
                    let mut static_label = filename.to_owned();
                    static_label.push_str(&format!(".{}", offset));
                    format!("@{}\nD=M\n@SP\nA=M\nM=D\n@SP\n@SP\nM=M+1\n", static_label)
                }
                MemorySegment::Temp => format!("@{}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", 5 + offset),
                MemorySegment::Pointer => {
                    if *offset == 0 {
                        format!("@THIS\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n")
                    } else {
                        format!("@THAT\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n")
                    }
                }

                _ => format!(
                    "@{}\nD=A\n@{}\nA=D+M\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n",
                    offset,
                    segment.to_label(),
                ),
            },

            Command::Pop { segment, offset } => match segment {
                MemorySegment::Static => {
                    let mut static_label = filename.to_owned();
                    static_label.push_str(&format!(".{}", offset));
                    format!("@SP\nM=M-1\nA=M\nD=M\n@{}\nM=D\n", static_label)
                }
                MemorySegment::Temp => format!("@SP\nM=M-1\nA=M\nD=M\n@{}\nM=D\n", 5 + offset),
                MemorySegment::Pointer => {
                    if *offset == 0 {
                        format!("@SP\nM=M-1\nA=M\nD=M\n@THIS\nM=D\n")
                    } else {
                        format!("@SP\nM=M-1\nA=M\nD=M\n@THAT\nM=D\n")
                    }
                }
                MemorySegment::Constant => {
                    panic!("Pop operation cannot be performed for a constant")
                }

                _ => format!(
                    "@{}\nD=M\n@13\nM=D\n@{}\nD=A\n@13\nM=D+M\n@SP\nM=M-1\nA=M\nD=M\n@13\nA=M\nM=D\n",
                    segment.to_label(),
                    offset,
                ),
            },

            Command::Add => format!("@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nM=D+M\n@SP\nM=M+1\n"),
            Command::Sub => format!("@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nM=M-D\n@SP\nM=M+1\n"),
            Command::Neg => format!("@SP\nM=M-1\nA=M\nM=-M\n@SP\nM=M+1\n"),

            Command::Not => format!("@SP\nM=M-1\nA=M\nM=!M\n@SP\nM=M+1\n"),
            Command::Or => format!("@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nM=D|M\n@SP\nM=M+1\n"),
            Command::And => format!("@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nM=D&M\n@SP\nM=M+1\n"),
            Command::Eq => {
                let (jump_start, jump_end) = jump_labels();
                format!(
                    "@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@{}\nD;JEQ\n@SP\nA=M\nM=0\n@{}\n0;JMP\n({})\n@SP\nA=M\nM=-1\n({})\n@SP\nM=M+1",
                    jump_start,
                    jump_end,
                    jump_start,
                    jump_end
                )
            }
            Command::Lt => {
                let (jump_start, jump_end) = jump_labels();
                format!(
                    "@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@{}\nD;JLT\n@SP\nA=M\nM=0\n@{}\n0;JMP\n({})\n@SP\nA=M\nM=-1\n({})\n@SP\nM=M+1",
                    jump_start,
                    jump_end,
                    jump_start,
                    jump_end
                )
            }
            Command::Gt => {
                let (jump_start, jump_end) = jump_labels();
                format!(
                    "@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n@{}\nD;JGT\n@SP\nA=M\nM=0\n@{}\n0;JMP\n({})\n@SP\nA=M\nM=-1\n({})\n@SP\nM=M+1",
                    jump_start,
                    jump_end,
                    jump_start,
                    jump_end
                )
            }

            _ => unimplemented!(),
        }
    }

    fn verify_offset(&self) {
        match self {
            Command::Push { segment, offset } | Command::Pop { segment, offset } => {
                match segment {
                    MemorySegment::Static => {
                        if *offset > 238 {
                            // RAM[16-255]
                            panic!("Received offset out of STATIC range (238 reg)  {}", offset);
                        }
                    }
                    MemorySegment::Temp => {
                        if *offset > 7 {
                            // RAM[5-12]
                            panic!("Received offset out of TEMP range (8 reg)  {}", offset);
                        }
                    }
                    MemorySegment::Pointer => {
                        if !(0..=1).contains(offset) {
                            panic!("POINTER offset can be either 0 or 1 | Received {}", offset);
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

impl MemorySegment {
    fn to_label(&self) -> &str {
        match self {
            MemorySegment::Local => "LCL",
            MemorySegment::Argument => "ARG",
            MemorySegment::This => "THIS",
            MemorySegment::That => "THAT",
            _ => panic!("Shoudln't have come here"),
        }
    }
}

impl FromStr for MemorySegment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segment = match s {
            "local" => MemorySegment::Local,
            "argument" => MemorySegment::Argument,
            "this" => MemorySegment::This,
            "that" => MemorySegment::That,
            "constant" => MemorySegment::Constant,
            "static" => MemorySegment::Static,
            "temp" => MemorySegment::Temp,
            "pointer" => MemorySegment::Pointer,
            _ => return Err(format!("Received unknown memory segment {}", s)),
        };

        Ok(segment)
    }
}

fn jump_labels() -> (String, String) {
    unsafe {
        NEXTJUMP += 1;
    }
    (
        format!("JUMP_START_{}", unsafe { NEXTJUMP }),
        format!("JUMP_END_{}", unsafe { NEXTJUMP }),
    )
}
