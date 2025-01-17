use std::{
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use compiler::{ast, bytecode, il};
use reader::{Reader, Sexpr};
use vm::{OpCode, OpCodeTable, Vm};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut il_compiler = il::Compiler::new();
    let mut ast_compiler = ast::Compiler::new();
    let mut vm: Vm<&Sexpr<'_>> = Vm::new();
    let mut opcode_table = OpCodeTable::new();

    lisp::compile_file(
        PathBuf::from("lib/bootstrap/bootstrap.lisp").as_path(),
        &mut il_compiler,
        &mut ast_compiler,
        &mut vm,
        &mut opcode_table,
    )?;

    lisp::compile_file(
        PathBuf::from("lib/native/decl/native.lisp").as_path(),
        &mut il_compiler,
        &mut ast_compiler,
        &mut vm,
        &mut opcode_table,
    )?;

    for arg in env::args().skip(1) {
        let path = PathBuf::from(arg);

        let mut opcode_table = OpCodeTable::new();

        lisp::compile_file(
            path.as_path(),
            &mut il_compiler,
            &mut ast_compiler,
            &mut vm,
            &mut opcode_table,
        )?;

        disasm(&opcode_table, 0);
    }

    Ok(())
}

fn disasm(opcode_table: &OpCodeTable<&Sexpr>, depth: usize) {
    let indent = "  ".repeat(depth);

    for opcode in opcode_table.opcodes() {
        println!("{indent}{opcode:?}");

        if let OpCode::Lambda { body, .. } = opcode {
            disasm(body, depth + 1)
        }
    }
}
