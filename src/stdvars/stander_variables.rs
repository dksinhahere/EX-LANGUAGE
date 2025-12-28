use crate::interpreter::error::RuntimeResult;
use crate::values::values::{Environment, Value};

pub(crate) fn define_std_vars(env: &mut Environment) -> RuntimeResult<()> {
    // -------------------------------------------------
    // Language / Runtime
    // -------------------------------------------------
    env.define_constant("__VERSION__", Value::String("0.1.0".into()))?;
    env.define_constant("__LANG__", Value::String("EX".into()))?;

    // -------------------------------------------------
    // OS / Architecture
    // -------------------------------------------------
    env.define_constant("__OS__", Value::String(std::env::consts::OS.into()))?;
    env.define_constant("__ARCH__", Value::String(std::env::consts::ARCH.into()))?;
    env.define_constant("__FAMILY__", Value::String(std::env::consts::FAMILY.into()))?;
    env.define_constant("__ABI__", Value::String("sysv".into()))?;

    // -------------------------------------------------
    // CPU information
    // -------------------------------------------------
    
    env.define_constant("__CPU_BITS__", Value::Int(usize::BITS as i128))?;

    env.define_constant(
        "__CPU_ENDIAN__",
        Value::String(
            if cfg!(target_endian = "little") {
                "little"
            } else {
                "big"
            }
            .into(),
        ),
    )?;

    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get() as i64)
        .unwrap_or(1);

    env.define_constant("__CPU_CORES__", Value::Int(cpu_count as i128))?;
    env.define_constant("__CPU_LOGICAL_CORES__", Value::Int(cpu_count as i128))?;
    env.define_constant("__CPU_CACHE_LINE__", Value::Int(64))?;


    // -------------------------------------------------
    // Memory
    // -------------------------------------------------
    env.define_constant("__PTR_SIZE__", Value::Int(std::mem::size_of::<usize>() as i128))?;
    env.define_constant("__PAGE_SIZE__", Value::Int(4096))?;
    env.define_constant("__WORD_SIZE__", Value::Int(std::mem::size_of::<usize>() as i128))?;
    env.define_constant("__MAX_INT__", Value::Int(i128::MAX))?;
    env.define_constant("__MIN_INT__", Value::Int(i128::MIN))?;

    // -------------------------------------------------
    // Time / Clock
    // -------------------------------------------------
    env.define_constant("__CLOCKS_PER_SEC__", Value::Int(1_000_000))?;
    env.define_constant("__HAS_MONOTONIC_CLOCK__", Value::Bool(true))?;
    env.define_constant("__HAS_RTC__", Value::Bool(true))?;
    env.define_constant("__TIMER_RESOLUTION_NS__", Value::Int(1))?;

    // -------------------------------------------------
    // File system / IO
    // -------------------------------------------------
    env.define_constant("__PATH_SEP__", Value::String(std::path::MAIN_SEPARATOR.to_string()))?;
    env.define_constant("__LINE_SEP__", Value::String("\n".into()))?;
    env.define_constant("__STDIN_FD__", Value::Int(0))?;
    env.define_constant("__STDOUT_FD__", Value::Int(1))?;
    env.define_constant("__STDERR_FD__", Value::Int(2))?;

    // -------------------------------------------------
    // Signals / Process
    // -------------------------------------------------
    env.define_constant("__MAX_PID__", Value::Int(4194304))?;
    env.define_constant("__HAS_SIGNALS__", Value::Bool(true))?;
    env.define_constant("__HAS_FORK__", Value::Bool(cfg!(unix)))?;
    env.define_constant("__HAS_THREADS__", Value::Bool(true))?;

    // -------------------------------------------------
    // Math / Floating-point hardware
    // -------------------------------------------------
    env.define_constant("__HAS_FPU__", Value::Bool(true))?;
    env.define_constant("__FLOAT_RADIX__", Value::Int(2))?;
    env.define_constant("__FLOAT_MANTISSA_BITS__", Value::Int(52))?;
    env.define_constant("__FLOAT_MAX__", Value::Float(f64::MAX))?;
    env.define_constant("__FLOAT_MIN__", Value::Float(f64::MIN))?;

    // -------------------------------------------------
    // Primitive numeric types
    // TypeCasting
    // -------------------------------------------------
    env.define_constant("__INT__", Value::String("INTEGER".into()))?;
    env.define_constant("__UINT__", Value::String("UINTEGER".into()))?;
    env.define_constant("__FLOAT__", Value::String("FLOAT".into()))?;
    env.define_constant("__BIGINT__", Value::String("BIG_INTEGER".into()))?;

    // -------------------------------------------------
    // Text / character
    // -------------------------------------------------
    env.define_constant("__STRING__", Value::String("STRING".into()))?;
    env.define_constant("__CHAR__", Value::String("CHARACTER".into()))?;

    // -------------------------------------------------
    // Boolean / null
    // -------------------------------------------------
    env.define_constant("__BOOL__", Value::String("BOOLEAN".into()))?;
    env.define_constant("__NIL__", Value::String("NIL".into()))?;



    Ok(())
}
