use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_contains(src=array, value=any) -> Bool
    pub(crate) fn array_contains(src: Value, value: Value) -> RuntimeResult<Value> {
        let arr = expect_array(src, "array_contains")?;
        Ok(Value::Bool(arr.iter().any(|v| *v == value)))
    }
}
