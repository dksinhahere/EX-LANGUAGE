use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::{expect_array, expect_int, resolve_index};

impl Interpreter {
    // array_get(src=array, idx=int) -> Value
    pub(crate) fn array_get(src: Value, idx: Value) -> RuntimeResult<Value> {
        let arr = expect_array(src, "array_get")?;
        let i = expect_int(idx, "array_get", "idx")?;
        let u = resolve_index(i, arr.len(), "array_get")?;
        Ok(arr[u].clone())
    }
}
