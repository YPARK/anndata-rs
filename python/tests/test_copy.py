import pytest
from hypothesis import given, example, settings, HealthCheck, strategies as st
from hypothesis.extra.numpy import *
from anndata_rs import AnnData, AnnDataSet, read

import math
import numpy as np
import pandas as pd
from pathlib import Path
import uuid
from scipy import sparse as sp
from scipy.sparse import csr_matrix, issparse, random

def h5ad(dir=Path("./")):
    dir.mkdir(exist_ok=True)
    return str(dir / Path(str(uuid.uuid4()) + ".h5ad"))

@given(
    x=arrays(
        integer_dtypes(endianness='=') | floating_dtypes(endianness='=', sizes=(32, 64)) |
        unsigned_integer_dtypes(endianness = '='),
        (47, 79),
    ),
    obsm = arrays(integer_dtypes(endianness='='), (47, 139)),
    obs = st.lists(st.integers(min_value=0, max_value=100000), min_size=47, max_size=47),
)
@settings(deadline=None, suppress_health_check = [HealthCheck.function_scoped_fixture])
def test_copy_anndata(x, obsm, obs, tmp_path):
    csr = csr_matrix(obsm)
    adata = AnnData(
        X=x,
        obs = dict(txt=obs),
        obsm = dict(X_pca=obsm, sparse=csr),
        filename = h5ad(tmp_path),
    )
    adata_copy = adata.copy(h5ad(tmp_path))

    np.testing.assert_array_equal(adata.X[:], adata_copy.X[:])
    np.testing.assert_array_equal(adata.obsm["X_pca"], adata_copy.obsm["X_pca"])
    np.testing.assert_array_equal(adata.obsm["sparse"].todense(), adata_copy.obsm["sparse"].todense())
    np.testing.assert_array_equal(adata.obs["txt"], adata_copy.obs["txt"])