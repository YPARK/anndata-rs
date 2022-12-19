use crate::data::instance::*;

use pyo3::prelude::*;
use anndata::data::{Shape, SelectInfo, SelectInfoElem};

pub fn to_select_info(ob: &PyAny, shape: &Shape) -> PyResult<SelectInfo> {
    let py = ob.py();
    let ndim = shape.ndim();
    if is_none_slice(py, ob)? {
        Ok(std::iter::repeat(SelectInfoElem::full()).take(ndim).collect())
    } else if ob.is_instance_of::<pyo3::types::PyTuple>()? {
        ob.iter()?.zip(shape.as_ref())
            .map(|(x, len)| to_select_elem(x?, *len))
            .collect()
    } else {
        todo!()
    }
}

pub fn to_select_elem(ob: &PyAny, length: usize) -> PyResult<SelectInfoElem> {
    let py = ob.py();
    let select = if let Ok(slice) = ob.downcast::<pyo3::types::PySlice>() {
        let s = slice.indices(length as i64)?;
        ndarray::Slice { 
            start: s.start,
            end: Some(s.stop),
            step: s.step,
        }.into()
    } else if is_none_slice(py, ob)? {
        SelectInfoElem::full()
    } else if ob.is_instance_of::<pyo3::types::PyInt>()? {
        ob.extract::<usize>()?.into()
    } else {
        let boolean_mask: PyResult<Vec<bool>> =
            ob.iter()?.map(|x| x.unwrap().extract()).collect();
        match boolean_mask {
            Ok(mask) => {
                if mask.len() == length {
                    boolean_mask_to_indices(mask.into_iter()).into()
                } else if mask.len() == 0 {
                    Vec::new().into()
                } else {
                    panic!("boolean mask dimension mismatched")
                }
            }
            _ => ob.iter()?.map(|x| x.unwrap().extract()).collect::<PyResult<Vec<usize>>>()?.into(),
        }
    };
    Ok(select)
}

fn boolean_mask_to_indices<I>(iter: I) -> Vec<usize>
where
    I: Iterator<Item = bool>,
{
    iter.enumerate()
        .filter_map(|(i, x)| if x { Some(i) } else { None })
        .collect()
}