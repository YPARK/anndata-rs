mod common;
use common::*;

use ndarray::Array2;
use proptest::prelude::*;
use anndata::*;
use anndata_hdf5::H5;

fn test_speacial_cases<T: AnnDataOp>(adata: T) {
    let arr = Array2::<i32>::zeros((0, 0));
    let arr2 = Array2::<i32>::zeros((10, 20));
    adata.set_x(&arr).unwrap();
    assert!(adata.add_obsm("test", &arr2).is_err());
}

/*
fn test_iterator<T: AnnDataOp>(adata: T) {
    let (csr, chunks) = rand_chunks(997, 20, 41);

    adata.set_x_from_iter(chunks)?;

    assert_eq!(adata.n_obs(), 997);
    assert_eq!(adata.n_vars(), 20);
    assert_eq!(adata.read_x::<CsrMatrix<i64>>()?.unwrap(), csr);

    adata.add_obsm_from_iter::<_, CsrMatrix<i64>>("key", adata.read_x_iter(111).map(|x| x.0))?;
    assert_eq!(adata.fetch_obsm::<CsrMatrix<i64>>("key")?.unwrap(), csr);
}
*/

fn test_io<T: AnnDataOp>(adata: T) {
    proptest!(ProptestConfig::with_cases(999), |(x in array_data_strat())| {
        adata.set_x(&x).unwrap();
        prop_assert_eq!(adata.read_x::<ArrayData>().unwrap().unwrap(), x);
        adata.del_x().unwrap();
    });
}

#[test]
fn test_speacial_cases_h5() {
    with_empty_adata::<H5, _, _>(|adata| test_speacial_cases(adata));
}

#[test]
fn test_io_h5() {
    with_empty_adata::<H5, _, _>(|adata| test_io(adata));
}