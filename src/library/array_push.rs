use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_push(src=array, value=any) -> array
    pub(crate) fn array_push(src: Value, value: Value) -> RuntimeResult<Value> {
        let mut arr = expect_array(src, "array_push")?;
        arr.push(value);
        Ok(Value::Array(arr))
    }
}
