use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::{expect_array, expect_int, resolve_index};

impl Interpreter {
    // array_set(src=array, idx=int, value=any) -> array
    pub(crate) fn array_set(src: Value, idx: Value, value: Value) -> RuntimeResult<Value> {
        let mut arr = expect_array(src, "array_set")?;
        let i = expect_int(idx, "array_set", "idx")?;
        let u = resolve_index(i, arr.len(), "array_set")?;
        arr[u] = value;
        Ok(Value::Array(arr))
    }
}
