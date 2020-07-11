use crate::asm::AssemblyLineParseError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "t", content = "d")]
pub enum Request {
    #[serde(rename = "u")]
    UploadCode(String),
}

#[derive(Debug, Serialize)]
#[serde(tag = "t", content = "d")]
pub enum Response {
    #[serde(rename = "u")]
    UploadCode {
        success: bool,
        errors: Option<Vec<CodeError>>,
    },
}

#[derive(Debug, Serialize)]
pub struct CodeError {
    pub line: usize,
    pub error: AssemblyLineParseError,
}
