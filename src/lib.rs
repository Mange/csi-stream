#[macro_use]
extern crate nom;

mod csi;
pub use csi::CSI;

mod parser;
use parser::Token;
pub use parser::text_with_csi;

#[derive(Debug, PartialEq)]
pub enum Value {
    Text(String),
    CSI(CSI),
}

impl<'a> From<Token<'a>> for Value {
    fn from(token: Token<'a>) -> Value {
        match token {
            Token::CSI(_, _, _) => Value::CSI(CSI::from(token)),
            Token::Text(bytes) => Value::Text(String::from_utf8_lossy(bytes).into_owned()),
        }
    }
}

pub fn parse_string(string: &str) -> Result<Vec<Value>, nom::IError> {
    let bytes = String::from(string).into_bytes();
    text_with_csi(&bytes)
        .to_full_result()
        .map(|tokens| tokens.into_iter().map(Value::from).collect())
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
