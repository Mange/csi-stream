#[macro_use]
extern crate nom;

use std::fmt;

mod parser;
use parser::{text_with_csi, Token};

#[derive(Debug, PartialEq)]
pub enum Value {
    Text(String),
    CSI(CSI),
}

#[derive(Debug, PartialEq)]
pub enum CSI {
    Unknown(String),
}

pub fn parse_string(string: &str) -> Result<Vec<Value>, nom::IError> {
    let bytes = String::from(string).into_bytes();
    text_with_csi(&bytes)
        .to_full_result()
        .map(|tokens| tokens.into_iter().map(Value::from).collect())
}

impl<'a> From<Token<'a>> for Value {
    fn from(token: Token<'a>) -> Value {
        match token {
            Token::CSI(parameters, intermediaries, final_byte) => {
                Value::CSI(CSI::unknown(parameters, intermediaries, final_byte))
            }
            Token::Text(bytes) => Value::Text(String::from_utf8_lossy(bytes).into_owned()),
        }
    }
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

impl fmt::Display for CSI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("\x1b[")?;
        match *self {
            CSI::Unknown(ref s) => f.write_str(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values_string(values: &Vec<Value>) -> String {
        use std::fmt::Write;
        let mut string = String::new();
        for value in values {
            match value {
                &Value::Text(ref s) => string.push_str(&s),
                &Value::CSI(ref csi) => write!(&mut string, "{}", csi).unwrap(),
            }
        }
        string
    }

    macro_rules! assert_parsing(
        ($input:expr, $expected:expr) => (
            let input = $input;
            let expected = $expected;
            let parsed = parse_string(input).expect("Could not parse input");

            assert_eq!(parsed, expected);

            let two_pass = values_string(&parsed);
            assert_eq!(&two_pass, input);
            assert_eq!(
                parse_string(&two_pass).expect("Could not parse stringified result (second pass)"),
                expected
            );
        );
    );

    #[test]
    fn it_parses_unknown_csi_tokens() {
        assert_parsing!(
            "This is an unknown CSI token: \x1b[?!~",
            vec![
                Value::Text(String::from("This is an unknown CSI token: ")),
                Value::CSI(CSI::Unknown(String::from("?!~"))),
            ]
        );
    }
}
