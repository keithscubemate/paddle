use std::rc::Rc;

use crate::eval::{env::BuiltinError, value::Value};
use anyhow::{Context, Result, bail};

pub fn cons(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        return Err(BuiltinError::WrongConsArgCount.into());
    };

    let head = &args.0;

    let Value::Cons(tail_pair) = &args.1 else {
        return Err(BuiltinError::WrongConsArgCount.into());
    };

    if let Value::Cons(_) = tail_pair.1 {
        return Err(BuiltinError::WrongConsArgCount.into());
    }

    let tail = &tail_pair.0;

    Ok(Value::Cons(Rc::new((head.clone(), tail.clone()))))
}

pub fn car(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = &args.1 {
        return Err(BuiltinError::WrongCarArgCount.into());
    };

    let pair = match &args.0 {
        Value::Nil => return Err(BuiltinError::CarOnEmptyList.into()),
        Value::Cons(pair) => pair,
        _ => return Err(BuiltinError::WrongCarArgType.into()),
    };

    Ok(pair.0.clone())
}

pub fn cdr(args: &Value) -> Result<Value> {
    let Value::Cons(args) = args else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = &args.1 {
        return Err(BuiltinError::WrongCdrArgCount.into());
    };

    let pair = match &args.0 {
        Value::Nil => return Err(BuiltinError::CdrOnEmptyList.into()),
        Value::Cons(pair) => pair,
        _ => return Err(BuiltinError::WrongCdrArgType.into()),
    };

    Ok(pair.1.clone())
}

pub fn list(args: &Value) -> Result<Value> {
    Ok(args.clone())
}

pub fn append(args: &Value) -> Result<Value> {
    let mut aiter = args.to_cons_iter();

    let sarg = aiter.next().context("must have 2 args")?;
    let oarg = aiter.next().context("must have 2 args")?;

    sarg.splice(oarg.clone())
}
