
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use crate::values::values::Value;

pub fn expect_array(value: Value, fname: &str) -> RuntimeResult<Vec<Value>> {
    match value {
        Value::Array(v) => Ok(v),
        other => Err(RuntimeError::custom(format!(
            "{} expects Array, got {}",
            fname,
            other.type_name()
        ))),
    }
}

pub fn expect_int(value: Value, fname: &str, arg: &str) -> RuntimeResult<i128> {
    match value {
        Value::Int(i) => Ok(i),
        other => Err(RuntimeError::custom(format!(
            "{} expects Int for '{}', got {}",
            fname,
            arg,
            other.type_name()
        ))),
    }
}

pub fn resolve_index(idx: i128, len: usize, fname: &str) -> RuntimeResult<usize> {
    let len_i = len as i128;
    let real = if idx < 0 { len_i + idx } else { idx };

    if real < 0 || real >= len_i {
        return Err(RuntimeError::custom(format!(
            "{} index out of bounds: idx={}, len={}",
            fname, idx, len
        )));
    }
    Ok(real as usize)
}
