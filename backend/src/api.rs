use crate::asm::AssemblyLineParseError;

enum Request {
    UploadCode { bot_id: usize, code: String },
}

enum Response {
    UploadCode {
        success: bool,
        errors: Option<Vec<AssemblyLineParseError>>,
    },
}
