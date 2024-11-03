use crate::{
    backend::*,
    data::{
        data_traits::*,
        slice::{SelectInfoElem, Shape},
    },
};

use anyhow::{bail, ensure, Result};
use ndarray::{arr0, Array, ArrayD, ArrayView, CowArray, Dimension, IxDyn};
use polars::series::Series;

#[derive(Debug, Clone, PartialEq)]
pub enum DynScalar {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
}

/// macro to implement `From` trait for `DynScalar`
macro_rules! impl_from_dynscalar {
    ($($from:ident, $to:ident),*) => {
        $(
            impl From<$from> for DynScalar {
                fn from(val: $from) -> Self {
                    DynScalar::$to(val)
                }
            }

            impl ReadData for $from {
                fn read<B: Backend>(container: &DataContainer<B>) -> Result<Self> {
                    let dataset = container.as_dataset()?;
                    match dataset.dtype()? {
                        ScalarType::$to => Ok(dataset.read_scalar()?),
                        _ => bail!("Cannot read $from"),
                    }
                }
            }

            impl WriteData for $from {
                fn data_type(&self) -> DataType {
                    DataType::Scalar(ScalarType::$to)
                }
                fn write<B: Backend, G: GroupOp<B>>(&self, location: &G, name: &str) -> Result<DataContainer<B>> {
                    let dataset = location.new_scalar_dataset(name, self)?;
                    let mut container = DataContainer::Dataset(dataset);
                    let encoding_type = if $from::DTYPE == ScalarType::String {
                        "string"
                    } else {
                        "numeric-scalar"
                    };
                    container.new_str_attr("encoding-type", encoding_type)?;
                    container.new_str_attr("encoding-version", "0.2.0")?;
                    Ok(container)
                }
            }
        )*
    };
}

impl_from_dynscalar!(
    i8, I8, i16, I16, i32, I32, i64, I64, u8, U8, u16, U16, u32, U32, u64, U64, f32,
    F32, f64, F64, bool, Bool, String, String
);

impl WriteData for DynScalar {
    fn data_type(&self) -> DataType {
        macro_rules! dtype {
            ($variant:ident, $exp:expr) => {
                DataType::Scalar(ScalarType::$variant)
            };
        }
        crate::macros::dyn_map1!(self, DynScalar, dtype)
    }

    fn write<B: Backend, G: GroupOp<B>>(
        &self,
        location: &G,
        name: &str,
    ) -> Result<DataContainer<B>> {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                $exp.write(location, name)
            };
        }
        crate::macros::dyn_map1!(self, DynScalar, fun)
    }
}

impl ReadData for DynScalar {
    fn read<B: Backend>(container: &DataContainer<B>) -> Result<Self> {
        let dataset = container.as_dataset()?;

        macro_rules! fun {
            ($variant:ident) => {
                DynScalar::I8(dataset.read_scalar()?)
            };
        }

        Ok(crate::macros::dyn_map0!(dataset.dtype()?, ScalarType, fun))
    }
}

/// A dynamic-typed array.
#[derive(Debug, Clone, PartialEq)]
pub enum DynArray {
    I8(ArrayD<i8>),
    I16(ArrayD<i16>),
    I32(ArrayD<i32>),
    I64(ArrayD<i64>),
    U8(ArrayD<u8>),
    U16(ArrayD<u16>),
    U32(ArrayD<u32>),
    U64(ArrayD<u64>),
    F32(ArrayD<f32>),
    F64(ArrayD<f64>),
    Bool(ArrayD<bool>),
    String(ArrayD<String>),
}

macro_rules! impl_dyn_array_convert {
    ($from_type:ty, $to_type:ident) => {
        impl<D: Dimension> From<Array<$from_type, D>> for DynArray {
            fn from(data: Array<$from_type, D>) -> Self {
                DynArray::$to_type(data.into_dyn())
            }
        }

        impl<D: Dimension> TryFrom<DynArray> for Array<$from_type, D> {
            type Error = anyhow::Error;
            fn try_from(v: DynArray) -> Result<Self, Self::Error> {
                match v {
                    DynArray::$to_type(data) => {
                        if let Some(n) = D::NDIM {
                            ensure!(
                                data.ndim() == n,
                                format!("Dimension mismatch: {} (in) != {} (out)", data.ndim(), n)
                            );
                        }
                        Ok(data.into_dimensionality::<D>()?)
                    }
                    _ => bail!(
                        "Cannot convert {:?} to {} ArrayD",
                        v.data_type(),
                        stringify!($from_type)
                    ),
                }
            }
        }
    };
}

impl_dyn_array_convert!(i8, I8);
impl_dyn_array_convert!(i16, I16);
impl_dyn_array_convert!(i32, I32);
impl_dyn_array_convert!(i64, I64);
impl_dyn_array_convert!(u8, U8);
impl_dyn_array_convert!(u16, U16);
impl_dyn_array_convert!(u32, U32);
impl_dyn_array_convert!(u64, U64);
impl_dyn_array_convert!(f32, F32);
impl_dyn_array_convert!(f64, F64);
impl_dyn_array_convert!(bool, Bool);
impl_dyn_array_convert!(String, String);

impl Into<Series> for DynArray {
    fn into(self) -> Series {
        match self {
            DynArray::I8(x) => x.iter().collect(),
            DynArray::I16(x) => x.iter().collect(),
            DynArray::I32(x) => x.iter().collect(),
            DynArray::I64(x) => x.iter().collect(),
            DynArray::U8(x) => x.iter().collect(),
            DynArray::U16(x) => x.iter().collect(),
            DynArray::U32(x) => x.iter().collect(),
            DynArray::U64(x) => x.iter().collect(),
            DynArray::F32(x) => x.iter().collect(),
            DynArray::F64(x) => x.iter().collect(),
            DynArray::Bool(x) => x.iter().collect(),
            DynArray::String(x) => x.iter().map(|x| x.as_str()).collect(),
        }
    }
}

impl WriteData for DynArray {
    fn data_type(&self) -> DataType {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                $exp.data_type()
            };
        }
        crate::macros::dyn_map1!(self, Self, fun)
    }

    fn write<B: Backend, G: GroupOp<B>>(
        &self,
        location: &G,
        name: &str,
    ) -> Result<DataContainer<B>> {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                $exp.write(location, name)
            };
        }

        crate::macros::dyn_map1!(self, Self, fun)
    }
}

impl ReadData for DynArray {
    fn read<B: Backend>(container: &DataContainer<B>) -> Result<Self> {
        container.as_dataset()?.read_dyn_array()
    }
}

impl HasShape for DynArray {
    fn shape(&self) -> Shape {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                $exp.shape().to_vec()
            };
        }

        crate::macros::dyn_map1!(self, DynArray, fun).into()
    }
}

impl ArrayOp for DynArray {
    fn get(&self, index: &[usize]) -> Option<DynScalar> {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                $exp.get(index).map(|x| x.clone().into())
            };
        }

        crate::macros::dyn_map1!(self, DynArray, fun)
    }

    fn select<S>(&self, info: &[S]) -> Self
    where
        S: AsRef<SelectInfoElem>,
    {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                ArrayOp::select($exp, info).into()
            };
        }
        crate::macros::dyn_map1!(self, DynArray, fun)
    }

    fn vstack<I: Iterator<Item = Self>>(iter: I) -> Result<Self> {
        let mut iter = iter.peekable();
        match iter.peek().unwrap() {
            DynArray::U8(_) => {
                ArrayD::<u8>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::U16(_) => {
                ArrayD::<u16>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::U32(_) => {
                ArrayD::<u32>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::U64(_) => {
                ArrayD::<u64>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::I8(_) => {
                ArrayD::<i8>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::I16(_) => {
                ArrayD::<i16>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::I32(_) => {
                ArrayD::<i32>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::I64(_) => {
                ArrayD::<i64>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::F32(_) => {
                ArrayD::<f32>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::F64(_) => {
                ArrayD::<f64>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::Bool(_) => {
                ArrayD::<bool>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
            DynArray::String(_) => {
                ArrayD::<String>::vstack(iter.map(|x| x.try_into().unwrap())).map(|x| x.into())
            }
        }
    }
}

impl WriteArrayData for DynArray {}
impl ReadArrayData for DynArray {
    fn get_shape<B: Backend>(container: &DataContainer<B>) -> Result<Shape> {
        Ok(container.as_dataset()?.shape().into())
    }

    fn read_select<B, S>(container: &DataContainer<B>, info: &[S]) -> Result<Self>
    where
        B: Backend,
        S: AsRef<SelectInfoElem>,
    {
        container.as_dataset()?.read_dyn_array_slice(info)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DynCowArray<'a> {
    I8(CowArray<'a, i8, IxDyn>),
    I16(CowArray<'a, i16, IxDyn>),
    I32(CowArray<'a, i32, IxDyn>),
    I64(CowArray<'a, i64, IxDyn>),
    U8(CowArray<'a, u8, IxDyn>),
    U16(CowArray<'a, u16, IxDyn>),
    U32(CowArray<'a, u32, IxDyn>),
    U64(CowArray<'a, u64, IxDyn>),
    F32(CowArray<'a, f32, IxDyn>),
    F64(CowArray<'a, f64, IxDyn>),
    Bool(CowArray<'a, bool, IxDyn>),
    String(CowArray<'a, String, IxDyn>),
}

impl From<DynScalar> for DynCowArray<'_> {
    fn from(scalar: DynScalar) -> Self {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                DynCowArray::$variant(arr0($exp).into_dyn().into())
            };
        }
        crate::macros::dyn_map1!(scalar, DynScalar, fun)
    }
}

impl DynCowArray<'_> {
    pub fn ndim(&self) -> usize {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                $exp.ndim()
            };
        }
        
        crate::macros::dyn_map1!(self, DynCowArray, fun)
    }

    pub fn shape(&self) -> Shape {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                $exp.shape().to_vec()
            };
        }

        crate::macros::dyn_map1!(self, DynCowArray, fun).into()
    }

    pub fn len(&self) -> usize {
        macro_rules! fun {
            ($variant:ident, $exp:expr) => {
                $exp.len()
            };
        }

        crate::macros::dyn_map1!(self, DynCowArray, fun)
    }
}

macro_rules! impl_dyn_cowarray_convert {
    ($from_type:ty, $to_type:ident) => {
        impl<D: Dimension> From<Array<$from_type, D>> for DynCowArray<'_> {
            fn from(data: Array<$from_type, D>) -> Self {
                DynCowArray::$to_type(data.into_dyn().into())
            }
        }

        impl<'a, D: Dimension> From<ArrayView<'a, $from_type, D>> for DynCowArray<'a> {
            fn from(data: ArrayView<'a, $from_type, D>) -> Self {
                DynCowArray::$to_type(data.into_dyn().into())
            }
        }

        impl<'a, D: Dimension> From<CowArray<'a, $from_type, D>> for DynCowArray<'a> {
            fn from(data: CowArray<'a, $from_type, D>) -> Self {
                DynCowArray::$to_type(data.into_dyn())
            }
        }

        impl<D: Dimension> TryFrom<DynCowArray<'_>> for Array<$from_type, D> {
            type Error = anyhow::Error;
            fn try_from(v: DynCowArray) -> Result<Self, Self::Error> {
                match v {
                    DynCowArray::$to_type(data) => {
                        let arr: ArrayD<$from_type> = data.into_owned();
                        if let Some(n) = D::NDIM {
                            ensure!(
                                arr.ndim() == n,
                                format!("Dimension mismatch: {} (in) != {} (out)", arr.ndim(), n)
                            );
                        }
                        Ok(arr.into_dimensionality::<D>()?)
                    }
                    _ => bail!(
                        "Cannot convert to {} ArrayD",
                        stringify!($from_type)
                    ),
                }
            }
        }
    };
}

impl_dyn_cowarray_convert!(i8, I8);
impl_dyn_cowarray_convert!(i16, I16);
impl_dyn_cowarray_convert!(i32, I32);
impl_dyn_cowarray_convert!(i64, I64);
impl_dyn_cowarray_convert!(u8, U8);
impl_dyn_cowarray_convert!(u16, U16);
impl_dyn_cowarray_convert!(u32, U32);
impl_dyn_cowarray_convert!(u64, U64);
impl_dyn_cowarray_convert!(f32, F32);
impl_dyn_cowarray_convert!(f64, F64);
impl_dyn_cowarray_convert!(bool, Bool);
impl_dyn_cowarray_convert!(String, String);

pub trait ArrayCast<T> {
    fn cast<D: Dimension>(self) -> Result<Array<T, D>>;
}

impl ArrayCast<usize> for DynArray {
    fn cast<D: Dimension>(self) -> Result<Array<usize, D>> {
        let out = match self {
            DynArray::U8(x) => x.mapv(|x| x as usize).into_dimensionality::<D>()?,
            DynArray::U16(x) => x.mapv(|x| x as usize).into_dimensionality::<D>()?,
            DynArray::U32(x) => x.mapv(|x| x as usize).into_dimensionality::<D>()?,
            DynArray::U64(x) => x.mapv(|x| x as usize).into_dimensionality::<D>()?,
            DynArray::I8(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::I16(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::I32(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::I64(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            v => bail!("Cannot cast {} array to usize", v.data_type()),
        };
        Ok(out)
    }
}

impl ArrayCast<f64> for DynArray {
    fn cast<D: Dimension>(self) -> Result<Array<f64, D>> {
        let out = match self {
            DynArray::U8(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::U16(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::U32(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::I8(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::I16(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::I32(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            DynArray::Bool(x) => x.mapv(|x| x.try_into().unwrap()).into_dimensionality::<D>()?,
            v => bail!("Cannot cast {} array to usize", v.data_type()),
        };
        Ok(out)
    }
}