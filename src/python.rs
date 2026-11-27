//! Bind native prefix and candidate traversal to Python byte vocabularies.
use crate::Grammar;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBytes, PyDict, PySet},
};

#[pyclass(frozen)]
struct NativeGrammar(Grammar);

#[pymethods]
impl NativeGrammar {
    #[new]
    fn new(schema: &str) -> PyResult<Self> {
        let schema =
            serde_json::from_str(schema).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Grammar::compile(&schema)
            .map(Self)
            .map_err(PyValueError::new_err)
    }
    fn accepts(&self, data: &[u8]) -> bool {
        self.0.accepts(data)
    }

    fn allowed_ids<'py>(
        &self,
        py: Python<'py>,
        prefix: &[u8],
        vocabulary: &Bound<'py, PyDict>,
        eos_id: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PySet>> {
        let state = self.0.prefix(prefix);
        let result = PySet::empty(py)?;
        for (id, piece) in vocabulary.iter() {
            if !id.eq(eos_id)? && state.permits(piece.downcast::<PyBytes>()?.as_bytes()) {
                result.add(id)?;
            }
        }
        if state.complete() {
            result.add(eos_id)?;
        }
        Ok(result)
    }
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<NativeGrammar>()
}
