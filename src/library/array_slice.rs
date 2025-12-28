use crate::interpreter::interpreter::Interpreter;
use crate::values::values::Value;
use crate::interpreter::error::RuntimeResult;
use crate::library::array_utils::{expect_array, expect_int};

impl Interpreter {
    // array_slice(src=array, start=int, end=int) -> array
    pub(crate) fn array_slice(src: Value, start: Value, end: Value) -> RuntimeResult<Value> {
        let arr = expect_array(src, "array_slice")?;
        let s = expect_int(start, "array_slice", "start")?;
        let e = expect_int(end, "array_slice", "end")?;

        let len = arr.len() as i128;

        let mut ss = if s < 0 { len + s } else { s };
        let mut ee = if e < 0 { len + e } else { e };

        if ss < 0 { ss = 0; }
        if ee < 0 { ee = 0; }
        if ss > len { ss = len; }
        if ee > len { ee = len; }
        if ee < ss { ee = ss; }

        Ok(Value::Array(arr[ss as usize..ee as usize].to_vec()))
    }
}
