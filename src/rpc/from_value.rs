use jsonrpc_core::Value;
use std::collections::HashMap;

pub trait FromValue {
    // Result<Self, expected thing>
    fn from(val: Value) -> Result<Self, &'static str>
    where
        Self: Sized;
}

macro_rules! fv_number {
    ($numtype:ident <- $thru:ident <- $method:ident | $expected:expr) => {
        impl FromValue for $numtype {
            #[allow(clippy::cast_possible_truncation)]
            #[allow(clippy::replace_consts)]
            fn from(val: Value) -> Result<Self, &'static str> {
                match val {
                    Value::Number(n) => n
                        .$method()
                        .and_then(|n| {
                            if n > (std::$numtype::MAX as $thru)
                                || n < (std::$numtype::MIN as $thru)
                            {
                                None
                            } else {
                                Some(n as $numtype)
                            }
                        })
                        .ok_or($expected),
                    _ => Err("a number"),
                }
            }
        }
    };
}

fv_number!(f32 <- f64 <- as_f64 | "a 32-bit float");
fv_number!(f64 <- f64 <- as_f64 | "a float");
fv_number!(i8 <- i64 <- as_i64 | "a signed 8-bit int");
fv_number!(i16 <- i64 <- as_i64 | "a signed 16-bit int");
fv_number!(i32 <- i64 <- as_i64 | "a signed 32-bit int");
fv_number!(i64 <- i64 <- as_i64 | "a signed int");
fv_number!(isize <- i64 <- as_i64 | "a signed int");
fv_number!(u8 <- u64 <- as_u64 | "an unsigned 8-bit int");
fv_number!(u16 <- u64 <- as_u64 | "an unsigned 16-bit int");
fv_number!(u32 <- u64 <- as_u64 | "an unsigned 32-bit int");
fv_number!(u64 <- u64 <- as_u64 | "an unsigned int");
fv_number!(usize <- u64 <- as_u64 | "an unsigned int");

impl<T: FromValue> FromValue for Option<T> {
    fn from(val: Value) -> Result<Self, &'static str> {
        Ok(match val {
            Value::Null => None,
            v => Some(<T as FromValue>::from(v)?),
        })
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from(val: Value) -> Result<Self, &'static str> {
        match val {
            Value::Array(vec) => {
                let mut fv = Self::with_capacity(vec.len());
                for v in vec {
                    fv.push(<T as FromValue>::from(v)?);
                }
                Ok(fv)
            }
            _ => Err("an array"),
        }
    }
}

impl<T: FromValue> FromValue for HashMap<String, T> {
    fn from(val: Value) -> Result<Self, &'static str> {
        match val {
            Value::Object(map) => {
                let mut fm = Self::with_capacity(map.len());
                for (k, v) in map {
                    fm.insert(k, <T as FromValue>::from(v)?);
                }
                Ok(fm)
            }
            _ => Err("an array"),
        }
    }
}

impl<T: FromValue, U: FromValue> FromValue for (T, U) {
    fn from(val: Value) -> Result<Self, &'static str> {
        match val {
            Value::Array(vec) => {
                if vec.len() == 2 {
                    let mut vec = vec.clone();
                    Ok((
                        <T as FromValue>::from(vec.remove(0))?,
                        <U as FromValue>::from(vec.remove(0))?,
                    ))
                } else {
                    Err("a two-item array")
                }
            }
            _ => Err("an array"),
        }
    }
}