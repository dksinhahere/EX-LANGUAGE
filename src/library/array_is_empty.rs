use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    pub(crate) fn array_is_empty(src: Value) -> RuntimeResult<Value> {
        let arr = expect_array(src, "array_is_empty")?;
        Ok(Value::Bool(arr.is_empty()))
    }
}
