use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::{expect_array, expect_int};

impl Interpreter {
    // array_insert(src=array, idx=int, value=any) -> array
    pub(crate) fn array_insert(src: Value, idx: Value, value: Value) -> RuntimeResult<Value> {
        let mut arr = expect_array(src, "array_insert")?;
        let i = expect_int(idx, "array_insert", "idx")?;

        let len = arr.len() as i128;
        let u = if i < 0 { len + i } else { i };
        if u < 0 || u > len {
            return Err(crate::interpreter::error::RuntimeError::custom(format!(
                "array_insert index out of bounds: idx={}, len={}",
                i, arr.len()
            )));
        }

        arr.insert(u as usize, value);
        Ok(Value::Array(arr))
    }
}
