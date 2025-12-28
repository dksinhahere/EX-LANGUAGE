use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_sort(src=array) -> array
    pub(crate) fn array_sort(src: Value) -> RuntimeResult<Value> {
        let mut arr = expect_array(src, "array_sort")?;

        // Ensure homogeneous comparable types (simple version)
        if arr.iter().all(|v| matches!(v, Value::Int(_))) {
            arr.sort_by_key(|v| if let Value::Int(i) = v { *i } else { 0 });
            return Ok(Value::Array(arr));
        }

        if arr.iter().all(|v| matches!(v, Value::UInt(_))) {
            arr.sort_by_key(|v| if let Value::UInt(u) = v { *u } else { 0 });
            return Ok(Value::Array(arr));
        }

        if arr.iter().all(|v| matches!(v, Value::Float(_))) {
            arr.sort_by(|a, b| {
                let af = if let Value::Float(x) = a { *x } else { 0.0 };
                let bf = if let Value::Float(x) = b { *x } else { 0.0 };
                af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal)
            });
            return Ok(Value::Array(arr));
        }

        if arr.iter().all(|v| matches!(v, Value::String(_))) {
            arr.sort_by(|a, b| {
                let as_ = if let Value::String(s) = a { s } else { "" };
                let bs_ = if let Value::String(s) = b { s } else { "" };
                as_.cmp(bs_)
            });
            return Ok(Value::Array(arr));
        }

        Err(RuntimeError::custom(
            "array_sort supports only arrays of Int/UInt/Float/String",
        ))
    }
}
