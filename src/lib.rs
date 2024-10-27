use pyo3::prelude::*;

mod parser;

#[pyfunction]
fn parse(markdown: String) -> PyResult<parser::Tokens> {
    parser::parse(markdown)
}

#[pymodule]
fn md0(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}
