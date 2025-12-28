use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_concat(a=array, b=array) -> array
    pub(crate) fn array_concat(a: Value, b: Value) -> RuntimeResult<Value> {
        let mut left = expect_array(a, "array_concat")?;
        let right = expect_array(b, "array_concat")?;
        left.extend(right);
        Ok(Value::Array(left))
    }
}
