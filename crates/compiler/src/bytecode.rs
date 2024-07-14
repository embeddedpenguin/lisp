use crate::il::{self, Il};
use core::fmt;
use gc::Gc;
use reader::Sexpr;
use vm::{OpCode, OpCodeTable};

#[derive(Clone, Debug)]
pub struct Error<'il, 'ast, 'sexpr, 'context> {
    il: &'il Il<'ast, 'sexpr, 'context>,
    message: String,
}

impl<'il, 'ast, 'sexpr, 'context> fmt::Display for Error<'il, 'ast, 'sexpr, 'context> {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl<'il, 'ast, 'sexpr, 'context> std::error::Error for Error<'il, 'ast, 'sexpr, 'context> {}

pub fn compile<'opcodes, 'il, 'ast, 'sexpr: 'static, 'context: 'static>(
    il: &'il Il<'ast, 'sexpr, 'context>,
    opcodes: &'opcodes mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    match il {
        Il::Lambda(lambda) => compile_lambda(lambda, opcodes),
        Il::Def(def) => compile_def(def, opcodes),
        Il::If(r#if) => compile_if(r#if, opcodes),
        Il::VarRef(varref) => compile_varref(varref, opcodes),
        Il::Constant(constant) => compile_constant(constant, opcodes),
        Il::List(list) => compile_list(list, opcodes),
        Il::FnCall(fncall) => compile_fncall(fncall, opcodes),
        Il::ArithmeticOperation(op) => compile_arithmetic_operation(op, opcodes),
        _ => todo!("{il:?}"),
    }
}

fn compile_varref<'il, 'ast, 'sexpr, 'context>(
    varref: &'il il::VarRef<'ast, 'sexpr, 'context>,
    opcodes: &mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    let op = match varref {
        il::VarRef::Local { index, .. } => OpCode::GetLocal(*index),
        il::VarRef::UpValue { index, .. } => OpCode::GetUpValue(*index),
        il::VarRef::Global { name, .. } => OpCode::GetGlobal(Gc::new(name.clone())),
    };

    opcodes.push(op, varref.source().source_sexpr());

    Ok(())
}

fn compile_constant<'il, 'ast, 'sexpr, 'context>(
    constant: &'il il::Constant<'ast, 'sexpr, 'context>,
    opcodes: &mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    let op = match constant {
        il::Constant::Symbol { symbol, .. } => OpCode::PushSymbol(Gc::new(symbol.clone())),
        il::Constant::String { string, .. } => OpCode::PushString(Gc::new(string.clone())),
        il::Constant::Char { char, .. } => OpCode::PushChar(*char),
        il::Constant::Int { int, .. } => OpCode::PushInt(*int),
        il::Constant::Bool { bool, .. } => OpCode::PushBool(*bool),
        il::Constant::Nil { .. } => OpCode::PushNil,
    };

    opcodes.push(op, constant.source().source_sexpr());

    Ok(())
}

fn compile_lambda<'opcodes, 'il, 'ast, 'sexpr: 'static, 'context: 'static>(
    lambda: &'il il::Lambda<'ast, 'sexpr, 'context>,
    opcodes: &'opcodes mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    let mut lambda_opcode_table = OpCodeTable::new();

    for expr in &lambda.body {
        compile(expr, &mut lambda_opcode_table)?;
    }

    lambda_opcode_table.push(OpCode::Return, lambda.source.source_sexpr());

    opcodes.push(
        OpCode::Lambda {
            arity: lambda.arity,
            body: Gc::new(lambda_opcode_table),
        },
        lambda.source.source_sexpr(),
    );

    for upvalue in &lambda.upvalues {
        opcodes.push(
            vm::OpCode::CreateUpValue(*upvalue),
            lambda.source.source_sexpr(),
        );
    }

    Ok(())
}

fn compile_if<'opcodes, 'il, 'ast, 'sexpr: 'static, 'context: 'static>(
    r#if: &'il il::If<'ast, 'sexpr, 'context>,
    opcodes: &'opcodes mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    let mut then_opcodes = OpCodeTable::new();
    let mut else_opcodes = OpCodeTable::new();

    compile(&r#if.predicate, opcodes)?;
    compile(&r#if.then, &mut then_opcodes)?;
    compile(&r#if.r#else, &mut else_opcodes)?;

    opcodes.push(
        OpCode::Branch(then_opcodes.len() + 1),
        r#if.source.source_sexpr(),
    );

    then_opcodes.push(
        OpCode::Jmp(else_opcodes.len() as isize),
        r#if.source.source_sexpr(),
    );

    opcodes.append(then_opcodes);
    opcodes.append(else_opcodes);

    Ok(())
}

fn compile_def<'opcodes, 'il, 'ast, 'sexpr: 'static, 'context: 'static>(
    def: &'il il::Def<'ast, 'sexpr, 'context>,
    opcodes: &'opcodes mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    compile(&def.body, opcodes)?;

    opcodes.push(
        OpCode::DefGlobal(Gc::new(def.parameter.name.clone())),
        def.source.source_sexpr(),
    );

    Ok(())
}

fn compile_arithmetic_operation<'opcodes, 'il, 'ast, 'sexpr: 'static, 'context: 'static>(
    arithmetic_op: &'il il::ArithmeticOperation<'ast, 'sexpr, 'context>,
    opcodes: &'opcodes mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    compile(&arithmetic_op.lhs, opcodes)?;
    compile(&arithmetic_op.rhs, opcodes)?;

    opcodes.push(
        match arithmetic_op.operator {
            il::ArithmeticOperator::Add => OpCode::Add,
            il::ArithmeticOperator::Sub => OpCode::Sub,
            il::ArithmeticOperator::Mul => OpCode::Mul,
            il::ArithmeticOperator::Div => OpCode::Div,
        },
        arithmetic_op.source.source_sexpr(),
    );

    Ok(())
}

fn compile_list<'opcodes, 'il, 'ast, 'sexpr: 'static, 'context: 'static>(
    list: &'il il::List<'ast, 'sexpr, 'context>,
    opcodes: &'opcodes mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    for expr in &list.exprs {
        compile(expr, opcodes)?;
    }

    opcodes.push(OpCode::List(list.exprs.len()), list.source.source_sexpr());

    Ok(())
}

fn compile_fncall<'opcodes, 'il, 'ast, 'sexpr: 'static, 'context: 'static>(
    fncall: &'il il::FnCall<'ast, 'sexpr, 'context>,
    opcodes: &'opcodes mut OpCodeTable<&'sexpr Sexpr<'context>>,
) -> Result<(), Error<'il, 'ast, 'sexpr, 'context>> {
    compile(&fncall.function, opcodes)?;

    for arg in &fncall.args {
        compile(arg, opcodes)?
    }

    opcodes.push(
        OpCode::Call(fncall.args.len()),
        fncall.source.source_sexpr(),
    );

    Ok(())
}
