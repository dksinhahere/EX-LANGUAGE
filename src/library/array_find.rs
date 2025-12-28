use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::expect_array;

impl Interpreter {
    // array_find(src=array, value=any) -> Int (index) or Nil
    pub(crate) fn array_find(src: Value, value: Value) -> RuntimeResult<Value> {
        let arr = expect_array(src, "array_find")?;
        for (i, v) in arr.iter().enumerate() {
            if *v == value {
                return Ok(Value::Int(i as i128));
            }
        }
        Ok(Value::Nil)
    }
}
