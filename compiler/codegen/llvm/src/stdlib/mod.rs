pub(crate) mod io;
pub(crate) mod fs;
pub(crate) mod json;
pub(crate) mod http;

use crate::LLVMGenerator;

impl LLVMGenerator {
    pub(crate) fn declare_stdlib(&mut self) {
        self.declare_io();
        self.declare_fs();
        self.declare_json();
        self.declare_http();
    }
}
