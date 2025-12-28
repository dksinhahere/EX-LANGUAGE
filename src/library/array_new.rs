use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;

impl Interpreter {
    // array_new() -> []
    pub(crate) fn array_new() -> RuntimeResult<Value> {
        Ok(Value::Array(Vec::new()))
    }
}
