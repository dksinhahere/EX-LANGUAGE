use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_len(src=array) -> Int
    pub(crate) fn array_len(src: Value) -> RuntimeResult<Value> {
        let arr = expect_array(src, "array_len")?;
        Ok(Value::Int(arr.len() as i128))
    }
}
