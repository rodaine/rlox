extern crate rlox;

fn main() {
    let mut c = rlox::chunk::Chunk::default();
    c.write_simple(123, rlox::chunk::OpCode::Return);
    c.write_simple(123, rlox::chunk::OpCode::Unknown);
    c.write_const(124, 1.23);
    c.write_simple(124, rlox::chunk::OpCode::Return);
    c.disassemble("test")
}
