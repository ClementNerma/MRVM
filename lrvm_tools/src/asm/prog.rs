//! The [`Program`] struct allows to represent a strongly-typed assembly program.
//! If the program builds, then it's guaranteed to be correct and does not need a runtime validation.

use super::{Instr, InstrDecodingError, ProgramWord};

/// Strongly-typed assembly program
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Program(pub Vec<ProgramWord>);

impl Program {
    /// Create an empty program
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create a program from a set of program words
    pub fn from(pwords: Vec<ProgramWord>) -> Self {
        Self(pwords)
    }

    /// Create a program from a set of instructions
    pub fn from_instr(instr: Vec<Instr>) -> Self {
        Self(instr.into_iter().map(ProgramWord::Instr).collect())
    }

    /// Get the number of words in the program
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Iterate over the program's instructions
    pub fn prog_words(&self) -> impl Iterator<Item = &ProgramWord> {
        self.0.iter()
    }

    /// Prepend an instruction at the beginning of the program
    pub fn prepend(&mut self, instr: ProgramWord) -> &mut Self {
        self.0.insert(0, instr);
        self
    }

    /// Prepend a set of instructions at the beginning of the program
    pub fn prepend_all(&mut self, instr: impl AsRef<[ProgramWord]>) -> &mut Self {
        let instr = instr.as_ref();

        let tail = self.0.len() - instr.len();
        self.0.extend(instr);
        self.0[instr.len()..].rotate_left(tail);

        self
    }

    /// Append an instruction at the end of the program
    pub fn append(&mut self, instr: ProgramWord) -> &mut Self {
        self.0.push(instr);
        self
    }

    /// Append a set of instructions at the end of the program
    pub fn append_all(&mut self, instr: impl AsRef<[ProgramWord]>) -> &mut Self {
        self.0.extend_from_slice(instr.as_ref());
        self
    }

    /// Disassemble a machine code into a program.
    /// In case of error, returns a tuple containing the faulty instruction's index along with the decoding error.
    /// Raw data can be forbidden to ensure strict checking of instructions.
    pub fn decode(
        prog: impl AsRef<[u8]>,
        forbid_raw: bool,
    ) -> Result<Self, (usize, InstrDecodingError)> {
        let prog = prog.as_ref();

        // Ensure the source code is aligned
        if prog.len() % 4 != 0 {
            return Err((0, InstrDecodingError::SourceNotMultipleOf4Bytes));
        }

        let mut out = vec![];

        // Decode all instructions (each instruction being on 4 bytes)
        for i in 0..prog.len() / 4 {
            let bytes = [
                prog[i * 4],
                prog[i * 4 + 1],
                prog[i * 4 + 2],
                prog[i * 4 + 3],
            ];

            let pword = match Instr::decode(bytes) {
                Ok(instr) => ProgramWord::Instr(instr),
                Err(err) => {
                    if forbid_raw {
                        return Err((i, err));
                    }

                    ProgramWord::Raw(bytes)
                }
            };

            out.push(pword);
        }

        Ok(Self::from(out))
    }

    /// Encode the program to folded bytes (list of 4-bytes slices)
    pub fn to_folded_bytes(&self) -> Vec<[u8; 4]> {
        self.prog_words().map(|pword| pword.encode()).collect()
    }

    /// Encode the program as a list of bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![];

        for pword in &self.0 {
            out.extend_from_slice(&pword.encode());
        }

        out
    }

    /// Encode the progrma as a list of words
    pub fn encode_words(&self) -> Vec<u32> {
        self.prog_words().map(|pword| pword.encode_word()).collect()
    }

    /// Convert the program to a LASM source code
    pub fn to_lasm(&self, annotate_instr_addr: bool) -> String {
        if !annotate_instr_addr {
            self.to_lasm_lines()
        } else {
            self.to_lasm_lines_annotated()
        }
        .join("\n")
    }

    /// Convert each line of the program to its LASM source code
    pub fn to_lasm_lines(&self) -> Vec<String> {
        self.prog_words().map(|pword| pword.to_lasm()).collect()
    }

    /// Convert each line of the program to its LASM source code with relative instructions address
    pub fn to_lasm_lines_annotated(&self) -> Vec<String> {
        let mut counter = 0;
        self.prog_words()
            .map(|instr| {
                let instr = format!("{:#010X}: {}", counter, instr.to_lasm());
                counter += 4;
                instr
            })
            .collect()
    }
}
