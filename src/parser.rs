macro_rules! byte_range(
    ($from:expr, $to:expr) => (
        ($from..($to+1)).collect::<Vec<u8>>().as_slice()
    );
);

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Text(&'a [u8]),
    CSI(Vec<char>, Vec<char>, char),
}

fn is_not_escape(b: u8) -> bool {
    b != 0x1b
}

named!(
    pub text_with_csi<Vec<Token>>,
    many0!(alt!(
            csi => { |(a, b, c)| Token::CSI(a, b, c) } |

            // Get rid of escape character in input in case it does not start a valid CSI sequence
            // to let the parser progress. Must be matched before the "any text" part.
            tag!("\x1b") => { |text| Token::Text(text) } |
            take_while!(is_not_escape) => { |text| Token::Text(text) }
        ))
);

named!(
    csi<(Vec<char>, Vec<char>, char)>,
    preceded!(
        csi_marker,
        tuple!(many0!(parameter), many0!(intermediary), final_byte)
    )
);
named!(csi_marker, tag!("\x1b["));
named!(parameter<char>, one_of!(byte_range!(0x30, 0x3f)));
named!(intermediary<char>, one_of!(byte_range!(0x20, 0x2f)));
named!(final_byte<char>, one_of!(byte_range!(0x40, 0x7e)));

#[cfg(test)]
mod test {
    use super::*;

    use nom::IResult::Done;

    mod raw {
        use super::*;

        #[test]
        fn it_parses_parameter() {
            assert_eq!(parameter(b"0"), Done(&[][..], '0'));
            assert_eq!(parameter(b"?"), Done(&[][..], '?'));
            assert_eq!(parameter(b";"), Done(&[][..], ';'));

            assert!(parameter(b"@").is_err());
            assert!(parameter(b"a").is_err());
        }

        #[test]
        fn it_parses_intermediary() {
            assert_eq!(intermediary(b"!"), Done(&[][..], '!'));
            assert_eq!(intermediary(b"/"), Done(&[][..], '/'));
            assert_eq!(intermediary(b"+"), Done(&[][..], '+'));

            assert!(intermediary(b"@").is_err());
            assert!(intermediary(b"5").is_err());
        }

        #[test]
        fn it_parses_final_byte() {
            assert_eq!(final_byte(b"@"), Done(&[][..], '@'));
            assert_eq!(final_byte(b"~"), Done(&[][..], '~'));
            assert_eq!(final_byte(b"m"), Done(&[][..], 'm'));

            assert!(final_byte(b">").is_err());
            assert!(final_byte(b" ").is_err());
        }

        #[test]
        fn it_parses_complete_csi_token() {
            assert_eq!(csi(b"\x1b[2A"), Done(&[][..], (vec!['2'], vec![], 'A')));
        }

        #[test]
        fn it_parses_text_without_csi_tokens() {
            assert_eq!(
                text_with_csi(b"Hello world"),
                Done(&[][..], vec![Token::Text(b"Hello world")])
            );
        }

        #[test]
        fn it_parses_csi_inside_text() {
            assert_eq!(
                text_with_csi(b"Hello \x1b[32mworld\x1b[0m"),
                Done(
                    &[][..],
                    vec![
                        Token::Text(b"Hello "),
                        Token::CSI(vec!['3', '2'], vec![], 'm'),
                        Token::Text(b"world"),
                        Token::CSI(vec!['0'], vec![], 'm'),
                    ]
                )
            );
        }

        #[test]
        fn it_ignores_partial_csi() {
            assert_eq!(
                text_with_csi(b"\"\x1b[%0\" is not a valid CSI"),
                Done(
                    &[][..],
                    vec![
                        Token::Text(b"\""),
                        Token::Text(b"\x1b"),
                        Token::Text(b"[%0\" is not a valid CSI"),
                    ]
                )
            );
        }

        #[test]
        fn it_works_on_edge_cases() {
            assert_eq!(text_with_csi(b""), Done(&[][..], vec![]));
            assert_eq!(
                text_with_csi(b"\x1b\x1b!"),
                Done(
                    &[][..],
                    vec![
                        Token::Text(b"\x1b"),
                        Token::Text(b"\x1b"),
                        Token::Text(b"!"),
                    ]
                )
            );

            assert_eq!(
                text_with_csi(b"\x1b[32m\x1b\x1b[0m"),
                Done(
                    &[][..],
                    vec![
                        Token::CSI(vec!['3', '2'], vec![], 'm'),
                        Token::Text(b"\x1b"),
                        Token::CSI(vec!['0'], vec![], 'm'),
                    ]
                )
            );
        }
    }

}
