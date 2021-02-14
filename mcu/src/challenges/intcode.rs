use core::convert::TryFrom;

pub enum ErrorKind {
    InvalidOpcode(u32),
    EndOfMemory,
    InvalidMemoryAddr(usize),
}

pub enum OpCode {
    Halt,
    Add { a: u32, b: u32, dst: usize },
    Multiply { a: u32, b: u32, dst: usize },
}

impl OpCode {
    pub fn length(&self) -> usize {
        match self {
            OpCode::Halt => 1,
            OpCode::Add { .. } | OpCode::Multiply { .. } => 4,
        }
    }
}

impl TryFrom<&[u32]> for OpCode {
    type Error = ErrorKind;

    fn try_from(value: &[u32]) -> Result<Self, Self::Error> {
        match value {
            [1, a, b, dst, ..] => Ok(OpCode::Add {
                a: *a,
                b: *b,
                dst: *dst as usize,
            }),
            [2, a, b, dst, ..] => Ok(OpCode::Multiply {
                a: *a,
                b: *b,
                dst: *dst as usize,
            }),
            [99, ..] => Ok(OpCode::Halt),
            [op, ..] => Err(ErrorKind::InvalidOpcode(*op)),
            [] => Err(ErrorKind::EndOfMemory),
        }
    }
}

pub struct IntCode<const N: usize> {
    memory: [u32; N],
    pc: usize,
    is_halted: bool,
}

impl<const N: usize> IntCode<N> {
    pub fn new(memory: [u32; N]) -> Self {
        Self {
            memory,
            pc: 0,
            is_halted: false,
        }
    }

    pub fn memory(&self) -> &[u32] {
        &self.memory
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn is_halted(&self) -> bool {
        self.is_halted
    }

    pub fn step(&mut self) -> Result<(), ErrorKind> {
        if self.is_halted {
            return Ok(());
        }

        let opcode = self
            .memory
            .get(self.pc..)
            .ok_or(ErrorKind::InvalidMemoryAddr(self.pc))
            .and_then(OpCode::try_from)?;

        match opcode {
            OpCode::Halt => self.is_halted = true,
            OpCode::Add { a, b, dst } => {
                *self
                    .memory
                    .get_mut(dst)
                    .ok_or(ErrorKind::InvalidMemoryAddr(dst))? = a + b;
                self.pc += opcode.length();
            }
            OpCode::Multiply { a, b, dst } => {
                *self
                    .memory
                    .get_mut(dst)
                    .ok_or(ErrorKind::InvalidMemoryAddr(dst))? = a * b;
                self.pc += opcode.length();
            }
        }

        Ok(())
    }
}
