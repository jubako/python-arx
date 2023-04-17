use arx::CommonEntry;
use jubako as jbk;
use libarx as arx;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyUnicode;
use std::path::PathBuf;

#[pyclass]
struct ContentAddress(jbk::ContentAddress);

#[pyclass]
struct Entry(arx::FullEntry);

#[pymethods]
impl Entry {
    fn __repr__(&self) -> String {
        match &self.0 {
            arx::Entry::File(e) => {
                format!("File({})", String::from_utf8(e.path().clone()).unwrap())
            }
            arx::Entry::Link(e) => {
                format!("Link({})", String::from_utf8(e.path().clone()).unwrap())
            }
            arx::Entry::Dir(_, e) => {
                format!("Dir({})", String::from_utf8(e.path().clone()).unwrap())
            }
        }
    }

    #[getter]
    fn idx(&self) -> PyResult<u32> {
        Ok(match &self.0 {
            arx::Entry::File(e) => e.idx().into_u32(),
            arx::Entry::Link(e) => e.idx().into_u32(),
            arx::Entry::Dir(_, e) => e.idx().into_u32(),
        })
    }

    #[getter]
    fn path(&self) -> PyResult<String> {
        Ok(match &self.0 {
            arx::Entry::File(e) => String::from_utf8(e.path().clone()).unwrap(),
            arx::Entry::Link(e) => String::from_utf8(e.path().clone()).unwrap(),
            arx::Entry::Dir(_, e) => String::from_utf8(e.path().clone()).unwrap(),
        })
    }

    #[getter]
    fn parent(&self) -> PyResult<Option<u32>> {
        let parent = match &self.0 {
            arx::Entry::File(e) => e.parent(),
            arx::Entry::Link(e) => e.parent(),
            arx::Entry::Dir(_, e) => e.parent(),
        };
        Ok(parent.map(|p| p.into_u32()))
    }

    #[getter]
    fn owner(&self) -> PyResult<u32> {
        Ok(match &self.0 {
            arx::Entry::File(e) => e.owner(),
            arx::Entry::Link(e) => e.owner(),
            arx::Entry::Dir(_, e) => e.owner(),
        })
    }

    #[getter]
    fn group(&self) -> PyResult<u32> {
        Ok(match &self.0 {
            arx::Entry::File(e) => e.group(),
            arx::Entry::Link(e) => e.group(),
            arx::Entry::Dir(_, e) => e.group(),
        })
    }

    #[getter]
    fn rights(&self) -> PyResult<u8> {
        Ok(match &self.0 {
            arx::Entry::File(e) => e.rights(),
            arx::Entry::Link(e) => e.rights(),
            arx::Entry::Dir(_, e) => e.rights(),
        })
    }

    #[getter]
    fn mtime(&self) -> PyResult<u64> {
        Ok(match &self.0 {
            arx::Entry::File(e) => e.mtime(),
            arx::Entry::Link(e) => e.mtime(),
            arx::Entry::Dir(_, e) => e.mtime(),
        })
    }

    fn content_size(&self) -> PyResult<u64> {
        match &self.0 {
            arx::Entry::File(e) => Ok(e.size().into_u64()),
            _ => Err(PyValueError::new_err("Not a file")),
        }
    }

    fn content_address(&self) -> PyResult<ContentAddress> {
        match &self.0 {
            arx::Entry::File(e) => Ok(ContentAddress(e.content())),
            _ => Err(PyValueError::new_err("Not a file")),
        }
    }

    fn link_target(&self) -> PyResult<String> {
        match &self.0 {
            arx::Entry::Link(e) => Ok(String::from_utf8(e.target().clone()).unwrap()),
            _ => Err(PyValueError::new_err("Not a link")),
        }
    }
}

#[pyclass(unsendable)]
struct Arx(arx::Arx);

impl std::ops::Deref for Arx {
    type Target = arx::Arx;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[pymethods]
impl Arx {
    #[new]
    fn py_new(path: &PyUnicode) -> PyResult<Self> {
        let path: PathBuf = path.extract()?;
        match arx::Arx::new(path) {
            Ok(a) => Ok(Self(a)),

            Err(_) => Err(PyValueError::new_err("Cannot create arx")),
        }
    }

    fn get_entry(&self, path: &PyUnicode) -> PyResult<Entry> {
        match self
            .0
            .get_entry::<arx::FullBuilder, PathBuf>(path.extract()?)
        {
            Ok(e) => Ok(Entry(e)),
            Err(_) => Err(PyValueError::new_err("Cannot get entry")),
        }
    }

    fn get_content<'py>(
        &self,
        py: Python<'py>,
        content: &ContentAddress,
    ) -> PyResult<&'py pyo3::types::PyBytes> {
        let reader = self.0.container.get_reader(content.0).unwrap();
        let mut flux = reader.create_flux_all();
        let read = |slice: &mut [u8]| Ok(flux.read_exact(slice).unwrap());
        pyo3::types::PyBytes::new_with(py, reader.size().into_usize(), read)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyarx(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Arx>()?;
    m.add_class::<Entry>()?;
    Ok(())
}
