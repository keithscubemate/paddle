use anyhow::{Result, bail};

use crate::eval::{env::BuiltinError, value::Value};

pub fn add(args: &Value) -> Result<Value> {
    if !matches!(args, Value::Cons(_) | Value::Nil) {
        bail!("should give me a list");
    }

    let mut num = 0.0;

    let mut hold = args;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                num += val;
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num))
}

pub fn min(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        return Err(BuiltinError::NoInitforMinus.into());
    };

    let mut num = match pair.0 {
        Value::Num(num) => num,
        Value::Nil => {
            return Err(BuiltinError::NoInitforMinus.into());
        }
        _ => {
            return Err(BuiltinError::ExpectedNumArg.into());
        }
    };

    let mut hold = &pair.1;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                num -= val;
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num))
}

pub fn mul(args: &Value) -> Result<Value> {
    if !matches!(args, Value::Cons(_) | Value::Nil) {
        return Err(BuiltinError::NoInitforDiv.into());
    }

    let mut num = 1.0;

    let mut hold = args;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                num *= val;
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num))
}

pub fn div(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        return Err(BuiltinError::NoInitforDiv.into());
    };

    let mut num = match pair.0 {
        Value::Num(num) => num,
        Value::Nil => {
            return Err(BuiltinError::NoInitforDiv.into());
        }
        _ => {
            return Err(BuiltinError::ExpectedNumArg.into());
        }
    };
    let mut hold = &pair.1;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                num /= val;
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Num(num))
}

pub fn lt(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        return Err(BuiltinError::BadLtArgTypes.into());
    };

    let Value::Num(mut num) = pair.0 else {
        return Err(BuiltinError::ExpectedNumArg.into());
    };
    let mut hold = &pair.1;
    let mut pass = true;

    while let Value::Cons(pair) = hold {
        match pair.0 {
            Value::Num(val) => {
                pass &= num < val;
                num = val
            }
            Value::Nil => break,
            _ => return Err(BuiltinError::ExpectedNumArg.into()),
        }
        hold = &pair.1;
    }

    Ok(Value::Bool(pass))
}

pub fn modulo(args: &Value) -> Result<Value> {
    let Value::Cons(pair) = args else {
        bail!("should give me a list");
    };

    let Value::Cons(pair2) = &pair.1 else {
        bail!("should give me a list");
    };

    if let Value::Cons(_) = pair2.1 {
        return Err(BuiltinError::BadModArgCount.into());
    }

    match (&pair2.0, &pair.0) {
        (Value::Num(last), Value::Num(penu)) => Ok(Value::Num(penu % last)),
        _ => Err(BuiltinError::BadModArgTypes.into()),
    }
}
