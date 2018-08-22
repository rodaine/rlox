extern crate byteorder;

use self::byteorder::{ByteOrder, NativeEndian};
use skip::SkipList;
use std::fmt;
use value::Value;

const MAX_8: usize = u8::max_value() as usize;
const MAX_16: usize = u16::max_value() as usize;
const MAX_24: usize = MAX_16 * 8;

#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    Unknown,
    Return,
    Constant8,
    Constant16,
    Constant24,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl OpCode {
    pub fn data_len(self) -> usize {
        use chunk::OpCode::*;

        match self {
            Unknown
            | Return
            | Negate
            | Add
            | Subtract
            | Multiply
            | Divide => 0,
            Constant8 => 1,
            Constant16 => 2,
            Constant24 => 3,
        }
    }
}

pub struct Instruction<'a> {
    pub op: OpCode,
    pub data: &'a [u8],
}

impl<'a> Instruction<'a> {
    pub fn new(op: OpCode, data: &'a [u8]) -> Self {
        Self { op, data }
    }

    pub fn len(&self) -> usize {
        1 + self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        use chunk::OpCode::*;

        match self {
            Unknown => 0,
            Return => 1,
            Constant8 => 2,
            Constant16 => 3,
            Constant24 => 4,
            Negate => 5,
            Add => 6,
            Subtract => 7,
            Multiply => 8,
            Divide => 9,
        }
    }
}

impl From<u8> for OpCode {
    fn from(b: u8) -> Self {
        use chunk::OpCode::*;

        match b {
            1 => Return,
            2 => Constant8,
            3 => Constant16,
            4 => Constant24,
            5 => Negate,
            6 => Add,
            7 => Subtract,
            8 => Multiply,
            9 => Divide,
            _ => Unknown,
        }
    }
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    lines: SkipList<usize>,
}

impl Chunk {
    pub fn write(&mut self, line: usize, op: OpCode, data: &[u8]) {
        debug_assert!(
            op.data_len() == data.len(),
            "invalid data length {} for op {:?}",
            data.len(),
            op
        );

        self.lines.push(self.code.len(), line);
        self.code.push(op.into());
        self.code.extend_from_slice(data);
    }

    pub fn write_simple(&mut self, line: usize, op: OpCode) {
        debug_assert!(op.data_len() == 0);
        self.write(line, op, &[])
    }

    pub fn write_const(&mut self, line: usize, constant: Value) {
        use chunk::OpCode::*;

        let idx = self.constants.len();
        self.constants.push(constant);

        match idx {
            _ if idx <= MAX_8 => self.write(line, Constant8, &[idx as u8]),
            _ if idx <= MAX_16 => {
                let mut enc = [0; 2];
                NativeEndian::write_u16(&mut enc, idx as u16);
                self.write(line, Constant16, &enc)
            }
            _ if idx <= MAX_24 => {
                let mut enc = [0; 3];
                NativeEndian::write_u24(&mut enc, idx as u32);
                self.write(line, Constant24, &enc)
            }
            _ => unreachable!(),
        }
    }

    pub fn read(&self, offset: usize) -> Option<Instruction> {
        if offset > self.code.len() {
            return None;
        }
        let op = OpCode::from(self.code[offset]);
        let data = &self.code[offset + 1..offset + 1 + op.data_len()];
        Some(Instruction::new(op, data))
    }

    pub fn read_index(data: &[u8]) -> usize {
        match data.len() {
            0 => 0,
            1 => data[0] as usize,
            2 => NativeEndian::read_u16(data) as usize,
            3 => NativeEndian::read_u24(data) as usize,
            _ => panic!("invalid index size {}", data.len()),
        }
    }

    pub fn read_const(&self, idx: usize) -> Value {
        *self.constants.get(idx).expect("invalid index")
    }

    pub fn disassemble(&self, name: &str) {
        eprint!("=== {} ===\n{:?}", name, self)
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut offset = 0;
        let mut line = 0;
        while offset < self.code.len() {
            let (o, l) = self.debug_inst(f, offset, line)?;
            writeln!(f)?;
            offset = o;
            line = l;
        }

        Ok(())
    }
}

impl Chunk {
    pub fn debug_inst(
        &self,
        f: &mut fmt::Formatter,
        offset: usize,
        last_line: usize,
    ) -> Result<(usize, usize), fmt::Error> {
        use chunk::OpCode::*;

        let inst = self.read(offset).unwrap();
        let line = self.lines.get(offset).cloned().unwrap_or(last_line);

        write!(f, "{:04}:  ", offset)?;

        if line == last_line {
            write!(f, "    |")?;
        } else {
            write!(f, "L{:04}", line)?;
        }

        write!(f, "  {:<10?}", inst.op)?;

        match inst.op {
            Return
            | Unknown
            | Negate
            | Add
            | Subtract
            | Multiply
            | Divide => {}
            Constant8 | Constant16 | Constant24 => {
                let idx = Self::read_index(inst.data);
                let val = self.read_const(idx);
                write!(f, "{:6}  ({:?})", idx, val)?;
            }
        };

        Ok((offset + inst.len(), line))
    }
}
