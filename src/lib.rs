use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug)]
pub struct VMTranslator<W: Write> {
    writer: BufWriter<W>,
    next_jump: u16,
    ret_idx: u16,
    filestem: String,
}

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
    /* Syntax: push / pop <segment> <offset> */
    Push(MemorySegment, u16),
    Pop(MemorySegment, u16),

    Add,
    Sub,
    Neg,

    Not,
    Or,
    And,
    Eq,
    Lt,
    Gt,

    /* Syntax: label / goto / if-goto <label_name> */
    Label(String),
    Goto(String),
    IfGoto(String),

    /* Syntax: function <function_name> <nVars - no. of local vars in the function> */
    Function(String, u16),
    /* Syntax: call <function_name> <nArgs - no. of arguments taken by the called function> */
    Call(String, u16),

    Return,
}

impl VMTranslator<File> {
    pub fn new(inpath: &Path) -> io::Result<Self> {
        let outpath = inpath.with_extension("asm");
        let outfile = File::create(outpath)?;
        let writer = BufWriter::new(outfile);
        let filestem = inpath
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap()
            .to_owned();

        Ok(VMTranslator {
            writer,
            next_jump: 0,
            ret_idx: 0,
            filestem,
        })
    }
}

impl<W: Write> VMTranslator<W> {
    pub fn write_asm(&mut self, command: Command) -> io::Result<()> {
        command.verify_offset();

        let asm = match command {
            Command::Push(segment, offset) => match segment {
                MemorySegment::Constant => format!("@{}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", offset),
                MemorySegment::Static => {
                    let static_label = format!("{}.{}", self.filestem, offset);
                    format!("@{}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", static_label)
                }
                MemorySegment::Temp => format!("@{}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", 5 + offset),
                MemorySegment::Pointer => {
                    if offset == 0 {
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

            Command::Pop(segment, offset) => match segment {
                MemorySegment::Static => {
                    let static_label = format!("{}.{}", self.filestem, offset);
                    format!("@SP\nM=M-1\nA=M\nD=M\n@{}\nM=D\n", static_label)
                }
                MemorySegment::Temp => format!("@SP\nM=M-1\nA=M\nD=M\n@{}\nM=D\n", 5 + offset),
                MemorySegment::Pointer => {
                    if offset == 0 {
                        format!("@SP\nM=M-1\nA=M\nD=M\n@THIS\nM=D\n")
                    } else {
                        format!("@SP\nM=M-1\nA=M\nD=M\n@THAT\nM=D\n")
                    }
                }
                MemorySegment::Constant => {
                    panic!("Pop operation cannot be performed for a constant")
                }

                _ => format!(
                    "@{}\nD=M\n@R13\nM=D\n@{}\nD=A\n@R13\nM=D+M\n\
                    @SP\nM=M-1\nA=M\nD=M\n@R13\nA=M\nM=D\n",
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
                let (jump_start, jump_end) = self.jump_labels();
                self.next_jump += 1;

                format!(
                    "@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n\
                    @{}\nD;JEQ\n@SP\nA=M\nM=0\n\
                    @{}\n0;JMP\n\
                    ({})\n@SP\nA=M\nM=-1\n\
                    ({})\n@SP\nM=M+1",
                    jump_start, jump_end, jump_start, jump_end
                )
            }
            Command::Lt => {
                let (jump_start, jump_end) = self.jump_labels();
                self.next_jump += 1;

                format!(
                    "@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n\
                    @{}\nD;JLT\n@SP\nA=M\nM=0\n\
                    @{}\n0;JMP\n\
                    ({})\n@SP\nA=M\nM=-1\n\
                    ({})\n@SP\nM=M+1",
                    jump_start, jump_end, jump_start, jump_end
                )
            }
            Command::Gt => {
                let (jump_start, jump_end) = self.jump_labels();
                self.next_jump += 1;

                format!(
                    "@SP\nM=M-1\nA=M\nD=M\n@SP\nM=M-1\nA=M\nD=M-D\n\
                    @{}\nD;JGT\n@SP\nA=M\nM=0\n\
                    @{}\n0;JMP\n\
                    ({})\n@SP\nA=M\nM=-1\n\
                    ({})\n@SP\nM=M+1",
                    jump_start, jump_end, jump_start, jump_end
                )
            }

            Command::Function(name, n_local_vars) => {
                let mut func_asm = format!("({})\n", name);
                for _ in 0..n_local_vars {
                    func_asm.push_str(&format!("@SP\nA=M\nM=0\n@SP\nM=M+1\n"));
                }

                func_asm
            }

            Command::Call(func_name, n_args) => self.translate_func_call(func_name, n_args),

            Command::Return => {
                /*
                 * Copy LCL to R13
                 * Store return addr in R14
                 * Move return val to arg 0
                 * Move SP to *ARG + 1
                 * Restore THIS, THAT, ARG, LCL pointers
                 * Uncoditional jump to return addr
                 */
                format!(
                    "@LCL\nD=M\n@R13\nM=D\n\
                    @5\nD=D-A\nA=D\nD=M\n@R14\nM=D\n\
                    @SP\nM=M-1\nA=M\nD=M\n@ARG\nA=M\nM=D\n\
                    @ARG\nD=M+1\n@SP\nM=D\n\
                    @R13\nD=M\n@1\nD=D-A\nA=D\nD=M\n@THAT\nM=D\n\
                    @R13\nD=M\n@2\nD=D-A\nA=D\nD=M\n@THIS\nM=D\n\
                    @R13\nD=M\n@3\nD=D-A\nA=D\nD=M\n@ARG\nM=D\n\
                    @R13\nD=M\n@4\nD=D-A\nA=D\nD=M\n@LCL\nM=D\n\
                    @R14\nA=M\n0;JMP\n"
                )
            }

            Command::Label(label) => format!("({})\n", label),
            Command::Goto(label) => format!("@{}\n0;JMP\n", label),
            Command::IfGoto(label) => format!("@SP\nM=M-1\nA=M\nD=M\n@{}\nD;JNE\n", label),
        };

        writeln!(self.writer, "{}", asm)?;

        Ok(())
    }

    pub fn translate_func_call(&mut self, func_name: String, n_args: u16) -> String {
        let ret_addr = format!("{}$ret.{}", func_name, self.ret_idx);
        self.ret_idx += 1;

        /* save current function frame */
        // return address in the ROM
        let mut call_asm = format!("@{}\nD=A\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", ret_addr);

        // recording segment pointers
        ["LCL", "ARG", "THIS", "THAT"].iter().for_each(|segment| {
            call_asm.push_str(&format!("@{}\nD=M\n@SP\nA=M\nM=D\n@SP\nM=M+1\n", segment));
        });

        // setting LCL to SP
        call_asm.push_str(&format!("@SP\nD=M\n@LCL\nM=D\n"));
        // setting arg 0 to first arg pushed onto stack
        call_asm.push_str(&format!(
            "@SP\nD=M\n@{}\nD=D-A\n@5\nD=D-A\n@ARG\nM=D\n",
            n_args
        ));

        call_asm.push_str(&format!("@{}\n0;JMP\n", func_name));
        call_asm.push_str(&format!("({})\n", ret_addr));

        call_asm
    }

    pub fn write_prelude(&mut self) -> io::Result<()> {
        writeln!(self.writer, "@256\nD=A\n@SP\nM=D\n\n")?;
        let sys_init = self.translate_func_call("Sys.init".into(), 0);
        writeln!(self.writer, "{}", sys_init)?;
        Ok(())
    }

    pub fn update_filestem(&mut self, curr_file: &PathBuf) {
        self.filestem = curr_file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap()
            .to_owned();
    }

    fn jump_labels(&self) -> (String, String) {
        (
            format!("JUMP_START_{}", self.next_jump),
            format!("JUMP_END_{}", self.next_jump),
        )
    }
}

pub fn parse(line: &str) -> Result<Command, String> {
    let parts: Vec<_> = line.split_whitespace().collect();
    let command = match parts[0] {
        "push" => Command::Push(
            MemorySegment::from_str(parts[1])?,
            parts[2].parse::<u16>().map_err(|e| e.to_string())?,
        ),

        "pop" => Command::Pop(
            MemorySegment::from_str(parts[1])?,
            parts[2].parse::<u16>().map_err(|e| e.to_string())?,
        ),

        "add" => Command::Add,
        "sub" => Command::Sub,
        "neg" => Command::Neg,
        "not" => Command::Not,
        "or" => Command::Or,
        "and" => Command::And,
        "eq" => Command::Eq,
        "lt" => Command::Lt,
        "gt" => Command::Gt,

        "label" => Command::Label(parts[1].to_owned()),
        "goto" => Command::Goto(parts[1].to_owned()),
        "if-goto" => Command::IfGoto(parts[1].to_owned()),

        "function" => Command::Function(
            parts[1].to_owned(),
            parts[2].parse::<u16>().map_err(|e| e.to_string())?,
        ),
        "call" => Command::Call(
            parts[1].to_owned(),
            parts[2].parse::<u16>().map_err(|e| e.to_string())?,
        ),
        "return" => Command::Return,

        _ => return Err(format!("Unknown command {}", parts[0])),
    };

    Ok(command)
}

impl Command {
    fn verify_offset(&self) {
        match self {
            Command::Push(segment, offset) | Command::Pop(segment, offset) => {
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

impl<W: Write> Drop for VMTranslator<W> {
    fn drop(&mut self) {
        if let Err(err) = self.writer.flush() {
            panic!("Couldn't flush writer: {}", err)
        }
    }
}
