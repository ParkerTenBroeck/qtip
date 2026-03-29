use std::{
    cell::UnsafeCell,
    collections::HashMap,
    path::{Path, PathBuf},
    pin::Pin,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SrcIdx(u32);

pub struct SourceMap {
    map: UnsafeCell<SourceMapInner>,
}

impl SourceMap {
    pub fn new(
        loader: impl FnMut(&Path) -> std::io::Result<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            map: UnsafeCell::new(SourceMapInner::new(loader)),
        }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn inner(&self) -> &mut SourceMapInner {
        unsafe { self.map.get().as_mut().unwrap_unchecked() }
    }

    pub fn load(&self, path: &Path) -> std::io::Result<&Source> {
        unsafe { self.inner() }.load(path)
    }

    pub fn get_idx(&self, idx: SrcIdx) -> Option<&Source> {
        unsafe { self.inner() }.get_idx(idx)
    }

    pub fn get_path(&self, path: &Path) -> Option<&Source> {
        unsafe { self.inner() }.get_path(path)
    }
}

type Loader = Box<dyn FnMut(&Path) -> std::io::Result<String> + Send + Sync>;

struct SourceMapInner {
    idx_map: Vec<Pin<Box<Source>>>,
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

    pub fn load(&mut self, path: &Path) -> std::io::Result<&Source> {
        let contents = (self.loader)(path)?;
        let idx = SrcIdx(self.idx_map.len() as u32);
        let source = Box::pin(Source {
            contents,
            path: path.to_path_buf(),
            idx,
        });

        self.idx_map.push(source);
        self.path_map.insert(path.to_path_buf(), idx);

        Ok(self.idx_map.last().unwrap())
    }

    pub fn get_idx(&self, idx: SrcIdx) -> Option<&Source> {
        Some(self.idx_map.get(idx.0 as usize)?)
    }

    pub fn get_path(&self, path: &Path) -> Option<&Source> {
        self.get_idx(*self.path_map.get(path)?)
    }
}

pub struct Source {
    pub contents: String,
    pub path: PathBuf,
    pub idx: SrcIdx,
}
