use pyo3::prelude::*;

mod parser;

#[pyfunction]
fn parse(markdown: String) -> PyResult<parser::Tokens> {
    parser::parse(markdown)
}

#[pyfunction]
fn tokens_to_html(tokens: parser::Tokens) -> PyResult<String> {
    parser::tokens_to_html(tokens)
}

#[pymodule]
fn md0(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(tokens_to_html, m)?)?;
    Ok(())
}
