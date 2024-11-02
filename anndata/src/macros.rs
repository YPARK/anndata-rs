macro_rules! dyn_map0 {
    ($scalar:expr, $enum:ident, $inner_macro:ident) => {
        match $scalar {
            $enum::I8 => $inner_macro!(I8),
            $enum::I16 => $inner_macro!(I16),
            $enum::I32 => $inner_macro!(I32),
            $enum::I64 => $inner_macro!(I64),
            $enum::U8 => $inner_macro!(U8),
            $enum::U16 => $inner_macro!(U16),
            $enum::U32 => $inner_macro!(U32),
            $enum::U64 => $inner_macro!(U64),
            $enum::Usize => $inner_macro!(Usize),
            $enum::F32 => $inner_macro!(F32),
            $enum::F64 => $inner_macro!(F64),
            $enum::Bool => $inner_macro!(Bool),
            $enum::String => $inner_macro!(String),
        }
    };
}

macro_rules! dyn_map1 {
    ($scalar:expr, $enum:ident, $inner_macro:ident) => {
        match $scalar {
            $enum::I8(_val) => $inner_macro!(I8, _val),
            $enum::I16(_val) => $inner_macro!(I16, _val),
            $enum::I32(_val) => $inner_macro!(I32, _val),
            $enum::I64(_val) => $inner_macro!(I64, _val),
            $enum::U8(_val) => $inner_macro!(U8, _val),
            $enum::U16(_val) => $inner_macro!(U16, _val),
            $enum::U32(_val) => $inner_macro!(U32, _val),
            $enum::U64(_val) => $inner_macro!(U64, _val),
            $enum::Usize(_val) => $inner_macro!(Usize, _val),
            $enum::F32(_val) => $inner_macro!(F32, _val),
            $enum::F64(_val) => $inner_macro!(F64, _val),
            $enum::Bool(_val) => $inner_macro!(Bool, _val),
            $enum::String(_val) => $inner_macro!(String, _val),
        }
    };
}

pub(crate) use {dyn_map0, dyn_map1};