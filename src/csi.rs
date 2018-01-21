use std::fmt;

use parser::Token;

#[derive(Debug, PartialEq)]
pub enum CSI {
    Unknown(String),
}

impl CSI {
    fn unknown(parameters: Vec<char>, intermediaries: Vec<char>, final_byte: char) -> CSI {
        let mut string = String::with_capacity(parameters.len() + intermediaries.len() + 1);
        for c in parameters.into_iter() {
            string.push(c);
        }
        for c in intermediaries.into_iter() {
            string.push(c);
        }
        string.push(final_byte);

        CSI::Unknown(string)
    }
}

impl<'a> From<Token<'a>> for CSI {
    fn from(token: Token) -> CSI {
        match token {
            Token::CSI(parameters, intermediaries, final_byte) => {
                CSI::unknown(parameters, intermediaries, final_byte)
            }
            _ => {
                panic!("CSI::from::<Token>() called with a Token that is not a Token::CSI. Called with a {:#?}", token);
            }
        }
    }
}

impl fmt::Display for CSI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("\x1b[")?;
        match *self {
            CSI::Unknown(ref s) => f.write_str(s),
        }
    }
}
