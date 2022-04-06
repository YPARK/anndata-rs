use crate::{
    anndata_trait::*,
    element::{
        MatrixElemOptional, DataFrameElem,
        ElemCollection, AxisArrays, Axis,
    },
    iterator::StackedChunkedMatrix,
};

use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use hdf5::{File, Result}; 
use polars::frame::DataFrame;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Clone)]
pub struct AnnData {
    pub(crate) file: File,
    pub n_obs: Arc<Mutex<usize>>,
    pub n_vars: Arc<Mutex<usize>>,
    pub x: MatrixElemOptional,
    pub obs: DataFrameElem,
    pub obsm: AxisArrays,
    pub obsp: AxisArrays,
    pub var: DataFrameElem,
    pub varm: AxisArrays,
    pub varp: AxisArrays,
    pub uns: ElemCollection,
}

impl AnnData {
    pub fn n_obs(&self) -> usize { *self.n_obs.lock().unwrap().deref() }

    pub(crate) fn set_n_obs(&self, n: usize) {
        *self.n_obs.lock().unwrap().deref_mut() = n;
    }

    pub fn n_vars(&self) -> usize { *self.n_vars.lock().unwrap().deref() }

    pub(crate) fn set_n_vars(&self, n: usize) {
        *self.n_vars.lock().unwrap().deref_mut() = n;
    }

    pub fn filename(&self) -> String { self.file.filename() }

    pub fn close(self) -> Result<()> { self.file.close() }

    pub fn set_x(&self, data: &Box<dyn DataPartialIO>) -> Result<()> {
        let n = self.n_obs();
        let m = self.n_vars();
        assert!(
            n == 0 || n == data.nrows(),
            "Number of observations mismatched, expecting {}, but found {}",
            n, data.nrows(),
        );
        assert!(
            m == 0 || m == data.ncols(),
            "Number of variables mismatched, expecting {}, but found {}",
            m, data.ncols(),
        );
        if !self.x.is_empty() { self.file.unlink("X")?; }
        self.x.insert(data.write(&self.file, "X")?)?;
        self.set_n_obs(data.nrows());
        self.set_n_vars(data.ncols());
        Ok(())
    }

    pub fn set_obs(&self, obs: &DataFrame) -> Result<()> {
        let n = self.n_obs();
        assert!(
            n == 0 || n == obs.nrows(),
            "Number of observations mismatched, expecting {}, but found {}",
            n, obs.nrows(),
        );
        if self.obs.is_empty() {
            self.obs.insert(obs.write(&self.file, "obs")?)?;
        } else {
            self.obs.update(obs);
        }
        self.set_n_obs(obs.nrows());
        Ok(())
    }

    pub fn set_obsm(&mut self, obsm: &HashMap<String, Box<dyn DataPartialIO>>) -> Result<()> {
        if self.file.group("obsm").is_ok() { self.file.unlink("obsm")?; }
        self.obsm = {
            let container = self.file.create_group("obsm")?;
            AxisArrays::new(container, Axis::Row, self.n_obs.clone())
        };
        for (key, data) in obsm.iter() {
            self.obsm.insert(key, data)?;
        }
        Ok(())
    }

    pub fn set_obsp(&mut self, obsp: &HashMap<String, Box<dyn DataPartialIO>>) -> Result<()> {
        if self.file.group("obsp").is_ok() { self.file.unlink("obsp")?; }
        self.obsp = {
            let container = self.file.create_group("obsp")?;
            AxisArrays::new(container, Axis::Both, self.n_obs.clone())
        };
        for (key, data) in obsp.iter() {
            self.obsp.insert(key, data)?;
        }
        Ok(())
    }

    pub fn set_var(&self, var: &DataFrame) -> Result<()> {
        let n = self.n_vars();
        assert!(
            n == 0 || n == var.nrows(),
            "Number of variables mismatched, expecting {}, but found {}",
            n, var.nrows(),
        );
        if self.var.is_empty() {
            self.var.insert(var.write(&self.file, "var")?)?;
        } else {
            self.var.update(var);
        }
        self.set_n_vars(var.nrows());
        Ok(())
    }

    pub fn set_varm(&mut self, varm: &HashMap<String, Box<dyn DataPartialIO>>) -> Result<()> {
        if self.file.group("varm").is_ok() { self.file.unlink("varm")?; }
        self.varm = {
            let container = self.file.create_group("varm")?;
            AxisArrays::new(container, Axis::Column, self.n_vars.clone())
        };
        for (key, data) in varm.iter() {
            self.varm.insert(key, data)?;
        }
        Ok(())
    }

    pub fn set_varp(&mut self, varp: &HashMap<String, Box<dyn DataPartialIO>>) -> Result<()> {
        if self.file.group("varp").is_ok() { self.file.unlink("varp")?; }
        self.varp = {
            let container = self.file.create_group("varp")?;
            AxisArrays::new(container, Axis::Both, self.n_vars.clone())
        };
        for (key, data) in varp.iter() {
            self.varp.insert(key, data)?;
        }
        Ok(())
    }

    pub fn set_uns(&mut self, uns: &HashMap<String, Box<dyn DataIO>>) -> Result<()> {
        if self.file.group("uns").is_ok() { self.file.unlink("uns")?; }
        self.uns = {
            let container = self.file.create_group("uns")?;
            ElemCollection::new(container)
        };
        for (key, data) in uns.iter() {
            self.uns.insert(key, data)?;
        }
        Ok(())
    }

    pub fn new(filename: &str, n_obs: usize, n_vars: usize) -> Result<Self> {
        let file = hdf5::File::create(filename)?;
        let n_obs = Arc::new(Mutex::new(n_obs));
        let n_vars = Arc::new(Mutex::new(n_vars));
        let obsm = {
            let container = file.create_group("obsm")?;
            AxisArrays::new(container, Axis::Row, n_obs.clone())
        };
        let obsp = {
            let container = file.create_group("obsp")?;
            AxisArrays::new(container, Axis::Both, n_obs.clone())
        };
        let varm = {
            let container = file.create_group("varm")?;
            AxisArrays::new(container, Axis::Column, n_vars.clone())
        };
        let varp = {
            let container = file.create_group("varp")?;
            AxisArrays::new(container, Axis::Both, n_vars.clone())
        };
        let uns = {
            let container = file.create_group("uns")?;
            ElemCollection::new(container)
        };
        Ok(Self { file, n_obs, n_vars,
            x: MatrixElemOptional::empty(),
            obs: DataFrameElem::empty(), obsm, obsp,
            var: DataFrameElem::empty(), varm, varp,
            uns,
        })
    }

    pub fn subset_obs(&self, idx: &[usize])
    {
        self.x.subset_rows(idx);
        self.obs.subset_rows(idx);
        self.obsm.subset(idx);
        self.obsp.subset(idx);
        self.set_n_obs(idx.len());
    }

    pub fn subset_var(&self, idx: &[usize])
    {
        self.x.subset_cols(idx);
        self.var.subset_cols(idx);
        self.varm.subset(idx);
        self.varp.subset(idx);
        self.set_n_vars(idx.len());
    }

    pub fn subset(&self, ridx: &[usize], cidx: &[usize])
    {
        self.x.subset(ridx, cidx);
        self.obs.subset_rows(ridx);
        self.obsm.subset(ridx);
        self.obsp.subset(ridx);
        self.var.subset_cols(cidx);
        self.varm.subset(cidx);
        self.varp.subset(cidx);
        self.set_n_obs(ridx.len());
        self.set_n_vars(cidx.len());
    }
}

pub struct AnnDataSet {
    pub anndatas: HashMap<String, AnnData>,
    pub n_obs: Arc<Mutex<usize>>,
    pub n_vars: Arc<Mutex<usize>>,
    pub obs: HashSet<String>,
    pub obsm: HashSet<String>,
    pub var: DataFrame,
}

impl AnnDataSet {
    pub fn new(anndatas: HashMap<String, AnnData>) -> Result<Self> {
        //if !anndatas.values().map(|x| x.var.read().unwrap().unwrap()[0]).all_equal() {
        //    panic!("var not equal");
        //}
        let var = DataFrame::new(vec![
            anndatas.values().next().unwrap().var.read().unwrap().unwrap()[0].clone()
        ]).unwrap();
        let n_vars = Arc::new(Mutex::new(var.height()));
        let n_obs = Arc::new(Mutex::new(anndatas.values().map(|x| x.n_obs()).sum()));
        let obs = intersections(anndatas.values().map(|x|
                x.obs.read().unwrap().unwrap().get_column_names().into_iter()
                .map(|s| s.to_string()).collect()).collect());
        let obsm = intersections(anndatas.values().map(
                |x| x.obsm.data.lock().unwrap().keys().map(Clone::clone).collect()).collect());

        Ok(Self { anndatas, n_vars, n_obs, obs, obsm, var })
    }

    pub fn n_obs(&self) -> usize { *self.n_obs.lock().unwrap().deref() }

    pub fn n_vars(&self) -> usize { *self.n_vars.lock().unwrap().deref() }

    pub fn chunked_x(&self, chunk_size: usize) -> StackedChunkedMatrix {
        StackedChunkedMatrix {
            matrices: self.anndatas.values().map(|x| x.x.chunked(chunk_size)).collect(),
            current_matrix_index: 0,
            n_mat: self.anndatas.len(),
        }
    }
}

fn intersections(mut sets: Vec<HashSet<String>>) -> HashSet<String> {
    {
        let (intersection, others) = sets.split_at_mut(1);
        let intersection = &mut intersection[0];
        for other in others {
            intersection.retain(|e| other.contains(e));
        }
    }
    sets[0].clone()
}