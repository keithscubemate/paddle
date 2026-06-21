use crate::eval::{env::BuiltinError, value::Value};
use anyhow::{Result, bail};

pub fn and(args: &Value) -> Result<Value> {
    let mut hold = args;
    let mut pass = true;

    while let Value::Cons(pair) = hold {
        pass &= pair.0.truthy();
        hold = &pair.1;
    }

    Ok(Value::Bool(pass))
}

pub fn val_or(args: &Value) -> Result<Value> {
    let mut hold = args;
    let mut pass = false;

    while let Value::Cons(pair) = hold {
        pass |= pair.0.truthy();
        hold = &pair.1;
    }

    Ok(Value::Bool(pass))
}

pub fn eq(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        bail!("should give me a list");
    };

    let Value::Cons(pair2) = &pair.1 else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = pair2.1 {
        return Err(BuiltinError::BadEqArgCount.into());
    }

    match (&pair.0, &pair2.0) {
        (Value::Num(last), Value::Num(penu)) => Ok(Value::Bool(penu == last)),
        (Value::Nil, Value::Nil) => Ok(Value::Bool(true)),
        (_, Value::Nil) | (Value::Nil, _) => Ok(Value::Bool(false)),
        (Value::Char(last), Value::Char(penu)) => Ok(Value::Bool(penu == last)),
        (Value::Char(byte), Value::Str(s))
        | (Value::Char(byte), Value::Symbol(s))
        | (Value::Str(s), Value::Char(byte))
        | (Value::Symbol(s), Value::Char(byte)) => {
            if s.len() != 1 {
                return Ok(Value::Bool(false));
            }

            let s = s.bytes().next().unwrap();
            Ok(Value::Bool(s == *byte))
        }
        (Value::Str(last), Value::Str(penu))
        | (Value::Symbol(last), Value::Str(penu))
        | (Value::Str(last), Value::Symbol(penu))
        | (Value::Symbol(last), Value::Symbol(penu)) => Ok(Value::Bool(penu == last)),
        _ => Ok(Value::Bool(false)),
    }
}

pub fn not(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = pair.1 {
        return Err(BuiltinError::WrongNotArgCount.into());
    }

    Ok(Value::Bool(!pair.0.truthy()))
}
