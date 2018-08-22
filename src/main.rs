extern crate rlox;

fn main() {
    use rlox::chunk::OpCode::*;
    use rlox::value::Value::*;

    let mut c = rlox::chunk::Chunk::default();
    c.write_const(1, Number(1.2));
    c.write_const(1, Number(3.4));
    c.write_simple(1, Add);
    c.write_const(1, Number(5.6));
    c.write_simple(1, Divide);
    c.write_simple(1, Negate);
    c.write_simple(1, Return);

    if let Err(e) = rlox::vm::VM::interpret(&c) {
        eprintln!("{:?}", e)
    }
}
