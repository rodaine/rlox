## rlox [![Build Status](https://travis-ci.org/rodaine/rlox.svg?branch=master)](https://travis-ci.org/rodaine/rlox)

_[Lox][lox] Interpreter/REPL written in Rust._

### Install

```bash
git checkout https://github.com/rodaine/rlox.git
cd rlox
cargo install

rlox            # starts the REPL
rlox script.lox # interprets the file
```

### Development

```bash
# linting - clippy is somewhat unstable
rustup run nightly cargo install clippy
rustup run nightly cargo clippy

# unit tests
cargo test --verbose
```

[lox]: http://www.craftinginterpreters.com/
