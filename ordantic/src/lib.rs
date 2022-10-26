use _pydantic_core::SchemaValidator;
use pyo3::create_exception;
use pyo3::prelude::*;

pub trait ToModelDict {
    fn to_model_dict<'py>(&self, py: Python<'py>) -> PyResult<PyObject>;
}

impl<T> ToModelDict for T
where
    T: ToPyObject,
{
    fn to_model_dict<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        Ok(self.to_object(py))
    }
}

#[pyclass]
pub struct ValidatorIterator {
    iter: Box<dyn Iterator<Item = SchemaValidator> + Send>,
}

impl ValidatorIterator {
    pub fn new(schema_validators: Vec<SchemaValidator>) -> Self {
        Self {
            iter: Box::new(schema_validators.into_iter()),
        }
    }
}

#[pymethods]
impl ValidatorIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<SchemaValidator> {
        slf.iter.next()
    }
}

/*
#[model]
struct ExampleModel {
    name: String,
    #[validate(range(min=1, max=10))]
    number: i64,
}

#[model]
struct ExampleModel2 {
    model: ExampleModel,
}
*/

//fn schema_to_py_dict(py: Python) -> PyDict {}

create_exception!(ordantic, OrdanticError, pyo3::exceptions::PyException);
