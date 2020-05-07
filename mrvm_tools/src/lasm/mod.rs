//! MRVM uses an assembly language called LASM (Lightweight Assembly).
//! This module allows to assemble LASM source code through the [CustomAsm](https://github.com/hlorenzi/customasm) library.

use crate::asm::{Program, InstrDecodingError};
use customasm::{FileServerMock, AssemblerState, RcReport};

static CUSTOMASM_HEADER: &'static str = include_str!("customasm.def");

/// Assemble a LASM source code to machine code.
/// Returns an error message in case of error.
pub fn assemble(source: &str) -> Result<Vec<u8>, String> {
    let mut src = String::from("#include \"header.lasm\"");
    src.push('\n');
    src.push_str(source);

    let mut fileserver = FileServerMock::new();
    fileserver.add("header.lasm", CUSTOMASM_HEADER);
    fileserver.add("src.lasm", src);

    let report = RcReport::new();

    let assemble = |report: RcReport, fileserver: &FileServerMock, filename: &str| -> Result<Vec<u8>, ()>
	{
		let mut asm = AssemblerState::new();
		asm.process_file(report.clone(), fileserver, filename)?;
		asm.wrapup(report)?;
		
		let output = asm.get_binary_output();
		Ok(output.generate_binary(0, output.len()))
    };
    
    assemble(report.clone(), &fileserver, "src.lasm").map_err(|_| {
        let mut err = vec![];
        report.print_all(&mut err, &fileserver);
        String::from_utf8(err).unwrap()
    })
}

/// Assemble a LASM source code to machine code and split it to words.
/// Returns an error message in case of error.
pub fn assemble_words(source: &str) -> Result<Vec<u32>, String> {
    assemble(source).map(|bytes| bytes_to_words(bytes))
}

/// Convert a list of bytes to a list of words
pub fn bytes_to_words(bytes: impl AsRef<[u8]>) -> Vec<u32> {
    let bytes = bytes.as_ref();

    let rem = bytes.len() % 4;
    let mut words = Vec::with_capacity(bytes.len() / 4 + if rem == 0 { 1 } else { 0 });
    let mut word = 0;

    for (i, byte) in bytes.iter().enumerate() {
        word += u32::from(*byte) << ((3 - (i % 4)) * 8);

        if i % 4 == 3 {
            words.push(word);
            word = 0;
        }
    }

    if rem != 0 {
        words.push(word << ((4 - rem) * 8));
    }

    words
}

/// Convert a LASM source code to a strongly-typed program
/// May fail because Program::decode() may fail if for instance there is raw data in the assembled program (strings for instance)
pub fn assemble_prog(source: &str) -> Result<Result<Program, (usize, InstrDecodingError)>, String> {
    Ok(Program::decode(assemble(source)?))
}
