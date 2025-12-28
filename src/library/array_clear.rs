use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_clear(src=array) -> array
    pub(crate) fn array_clear(src: Value) -> RuntimeResult<Value> {
        let _ = expect_array(src, "array_clear")?;
        Ok(Value::Array(Vec::new()))
    }
}
