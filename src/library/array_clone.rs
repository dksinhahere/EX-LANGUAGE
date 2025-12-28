use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_clone(src=array) -> array
    pub(crate) fn array_clone(src: Value) -> RuntimeResult<Value> {
        let arr = expect_array(src, "array_clone")?;
        Ok(Value::Array(arr))
    }
}
