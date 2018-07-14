extern crate byteorder;

use std::fmt;
use value::Value;
use skip::SkipList;
use self::byteorder::{ByteOrder, NativeEndian};

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
}

impl OpCode {
    pub fn data_len(self) -> usize {
        use chunk::OpCode::*;

        match self {
            Unknown | Return => 0,
            Constant8 => 1,
            Constant16 => 2,
            Constant24 => 3,
        }
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
        debug_assert!(op.data_len() == data.len(),
                      "invalid data length {} for op {:?}", data.len(), op);

        self.lines.push(self.code.len(), line);
        self.code.push(op.into());
        self.code.extend_from_slice(data);
    }

    pub fn write_simple(&mut self, line: usize, op: OpCode) { self.write(line, op, &[]) }

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

    pub fn read(&self, offset: usize) -> (OpCode, &[u8]) {
        let op = OpCode::from(*self.code.get(offset).expect("invalid offset"));
        let data = &self.code[offset + 1..offset + 1 + op.data_len()];
        (op, data)
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

    pub fn disassemble(&self, name: &str) { eprint!("=== {} ===\n{:?}", name, self) }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut offset = 0;
        let mut line = 0;
        while offset < self.code.len() {
            let (o, l) = self.debug_inst(f, offset, line)?;
            offset = o;
            line = l;
        }

        Ok(())
    }
}

impl Chunk {
    fn debug_inst(&self, f: &mut fmt::Formatter, offset: usize, last_line: usize) -> Result<(usize, usize), fmt::Error> {
        use chunk::OpCode::*;

        let (op, data) = self.read(offset);
        let line = self.lines.get(offset)
            .cloned().unwrap_or(last_line);

        write!(f, "{:04}:  ", offset)?;

        if line == last_line {
            write!(f, "    |")?;
        } else {
            write!(f, "L{:04}", line)?;
        }

        write!(f, "  {:<10?}", op)?;

        match op {
            Return | Unknown => writeln!(f)?,
            Constant8 | Constant16 | Constant24 => {
                let idx = Self::read_index(data);
                let val = self.read_const(idx);
                writeln!(f, "{:6}  ({:?})", idx, val)?;
            }
        };

        Ok((offset + 1 + data.len(), line))
    }
}
