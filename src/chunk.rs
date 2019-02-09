extern crate byteorder;

use self::byteorder::{ByteOrder, NativeEndian};
use crate::skip::SkipList;
use std::fmt;
use crate::value::Value;

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
    True,
    False,
    Nil,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal8,
    DefineGlobal16,
    DefineGlobal24,
    GetGlobal8,
    GetGlobal16,
    GetGlobal24,
    SetGlobal8,
    SetGlobal16,
    SetGlobal24,
}

impl OpCode {
    pub fn data_len(self) -> usize {
        use crate::chunk::OpCode::*;

        match self {
            Constant8 | DefineGlobal8 | GetGlobal8 | SetGlobal8 => 1,
            Constant16 | DefineGlobal16 | GetGlobal16 | SetGlobal16 => 2,
            Constant24 | DefineGlobal24 | GetGlobal24 | SetGlobal24 => 3,
            _ => 0
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
        use crate::chunk::OpCode::*;

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
            True => 10,
            False => 11,
            Nil => 12,
            Not => 13,
            Equal => 14,
            Greater => 15,
            Less => 16,
            Print => 17,
            Pop => 18,
            DefineGlobal8 => 19,
            DefineGlobal16 => 20,
            DefineGlobal24 => 21,
            GetGlobal8 => 22,
            GetGlobal16 => 23,
            GetGlobal24 => 24,
            SetGlobal8 => 25,
            SetGlobal16 => 26,
            SetGlobal24 => 27,
        }
    }
}

impl From<u8> for OpCode {
    fn from(b: u8) -> Self {
        use crate::chunk::OpCode::*;

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
            10 => True,
            11 => False,
            12 => Nil,
            13 => Not,
            14 => Equal,
            15 => Greater,
            16 => Less,
            17 => Print,
            18 => Pop,
            19 => DefineGlobal8,
            20 => DefineGlobal16,
            21 => DefineGlobal24,
            22 => GetGlobal8,
            23 => GetGlobal16,
            24 => GetGlobal24,
            25 => SetGlobal8,
            26 => SetGlobal16,
            27 => SetGlobal24,
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
        self.write(line, op, &[])
    }

    pub fn make_const(&mut self, constant: Value) -> usize {
        let idx = self.constants.len();
        self.constants.push(constant);
        idx
    }

    pub fn write_const(&mut self, line: usize, constant: Value) -> usize {
        use crate::chunk::OpCode::*;
        let idx = self.make_const(constant);
        self.write_idx(line, &[Constant8, Constant16, Constant24], idx);
        idx
    }

    pub fn write_idx(&mut self, line: usize, ops: &[OpCode; 3], idx: usize) {
        match idx {
            x if x <= MAX_8 => self.write(line, ops[0], &[x as u8]),
            x if x <= MAX_16 => {
                let mut enc = [0; 2];
                NativeEndian::write_u16(&mut enc, x as u16);
                self.write(line, ops[1], &enc);
            }
            x if x < MAX_24 => {
                let mut enc = [0; 3];
                NativeEndian::write_u24(&mut enc, x as u32);
                self.write(line, ops[2], &enc);
            }
            _ => panic!("usize value overflow: {}", idx),
        }
    }

    pub fn read(&self, offset: usize) -> Option<Instruction> {
        if offset >= self.code.len() {
            return None;
        }
        let op = OpCode::from(self.code[offset]);
        let data = &self.code[offset + 1..offset + 1 + op.data_len()];
        Some(Instruction::new(op, data))
    }

    pub fn read_const(&self, idx: usize) -> Value {
        self.constants.get(idx).unwrap().clone()
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
        use crate::chunk::OpCode::*;

        let inst = self.read(offset).unwrap();
        let line = self.lines.get(offset).cloned().unwrap_or(last_line);

        write!(f, "{:04}:L{:04}  {:<10}  ", offset, line, format!("{:?}", inst.op))?;

        match inst.op {
            Constant8 | Constant16 | Constant24 |
            DefineGlobal8 | DefineGlobal16 | DefineGlobal24 |
            GetGlobal8 | GetGlobal16 | GetGlobal24 |
            SetGlobal8 | SetGlobal16 | SetGlobal24 => {
                let idx = bytes_to_usize(inst.data);
                let val = self.read_const(idx);
                write!(f, "#{:<6} {:<30}", idx, format!("{:?}", val))?;
            }
            _ => {
                write!(f, "                                      ")?;
            }
        };

        Ok((offset + inst.len(), line))
    }
}

pub fn bytes_to_usize(bytes: &[u8]) -> usize {
    match bytes.len() {
        1 => bytes[0] as usize,
        2 => NativeEndian::read_u16(bytes) as usize,
        3 => NativeEndian::read_u24(bytes) as usize,
        _ => panic!("invalid data size {}", bytes.len()),
    }
}
