use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_reverse(src=array) -> array
    pub(crate) fn array_reverse(src: Value) -> RuntimeResult<Value> {
        let mut arr = expect_array(src, "array_reverse")?;
        arr.reverse();
        Ok(Value::Array(arr))
    }
}
