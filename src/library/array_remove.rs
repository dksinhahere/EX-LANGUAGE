use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::{expect_array, expect_int, resolve_index};

impl Interpreter {
    // array_remove(src=array, idx=int) -> array
    pub(crate) fn array_remove(src: Value, idx: Value) -> RuntimeResult<Value> {
        let mut arr = expect_array(src, "array_remove")?;
        let i = expect_int(idx, "array_remove", "idx")?;
        let u = resolve_index(i, arr.len(), "array_remove")?;
        arr.remove(u);
        Ok(Value::Array(arr))
    }
}
