use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SrcIdx(u32);

#[derive(Clone)]
pub struct SourceMap {
    map: Arc<RwLock<SourceMapInner>>,
}

impl SourceMap {
    pub fn new(
        loader: impl FnMut(&Path) -> std::io::Result<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            map: Arc::new(RwLock::new(SourceMapInner::new(loader))),
        }
    }

    pub fn load(&mut self, path: &Path) -> std::io::Result<Arc<Source>> {
        self.map.write().unwrap().load(path)
    }

    pub fn get_idx(&self, idx: SrcIdx) -> Option<Arc<Source>> {
        self.map.read().unwrap().get_idx(idx)
    }

    pub fn get_path(&self, path: &Path) -> Option<Arc<Source>> {
        self.map.read().unwrap().get_path(path)
    }
}

type Loader = Box<dyn FnMut(&Path) -> std::io::Result<String> + Send + Sync>;

pub struct SourceMapInner {
    idx_map: Vec<Arc<Source>>,
    path_map: HashMap<PathBuf, SrcIdx>,
    loader: Loader,
}

impl SourceMapInner {
    pub fn new(
        loader: impl FnMut(&Path) -> std::io::Result<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            idx_map: vec![],
            path_map: Default::default(),
            loader: Box::new(loader),
        }
    }

    pub fn load(&mut self, path: &Path) -> std::io::Result<Arc<Source>> {
        let contents = (self.loader)(path)?;
        let idx = SrcIdx(self.idx_map.len() as u32);
        let source = Arc::new(Source {
            contents,
            path: path.to_path_buf(),
            idx,
        });

        self.idx_map.push(source.clone());
        self.path_map.insert(path.to_path_buf(), idx);

        Ok(source)
    }

    pub fn get_idx(&self, idx: SrcIdx) -> Option<Arc<Source>> {
        self.idx_map.get(idx.0 as usize).cloned()
    }

    pub fn get_path(&self, path: &Path) -> Option<Arc<Source>> {
        self.get_idx(*self.path_map.get(path)?)
    }
}

pub struct Source {
    pub contents: String,
    pub path: PathBuf,
    pub idx: SrcIdx,
}