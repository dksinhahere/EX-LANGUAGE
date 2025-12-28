use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::{RuntimeError, RuntimeResult};
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_pop(src=array) -> Value
    pub(crate) fn array_pop(src: Value) -> RuntimeResult<Value> {
        let mut arr = expect_array(src, "array_pop")?;
        arr.pop().ok_or_else(|| RuntimeError::custom("array_pop on empty array"))
    }
}
