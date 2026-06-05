//! Borrowed-`&str` tokeniser for citation atoms (plan 03 T1).
//!
//! What: splits a citation string into words, integers, brackets, and
//! punctuation, each carrying its byte range.
//! How: a single forward pass; no allocation for token text (`&str` slices —
//! Rust standards: borrowed data through the parser).
//! Depends on: nothing.

/// The kind of a lexed token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenKind {
    /// A word: letters, optionally with embedded full stops or
    /// apostrophes/hyphens (`Mabo`, `C.L.R.`, `O'Brien`, `Ex-parte`, `v.`).
    Word,
    /// An unsigned integer run of ASCII digits.
    Int,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `[`
    OpenSquare,
    /// `]`
    CloseSquare,
    /// `,`
    Comma,
    /// `;`
    Semicolon,
    /// Any other non-whitespace character.
    Other,
}

/// One token: kind + the byte range it covers in the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

/// Tokenise `src`. Whitespace separates tokens and is not emitted.
pub(crate) fn lex(src: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        let kind = match b {
            b'(' => TokenKind::OpenParen,
            b')' => TokenKind::CloseParen,
            b'[' => TokenKind::OpenSquare,
            b']' => TokenKind::CloseSquare,
            b',' => TokenKind::Comma,
            b';' => TokenKind::Semicolon,
            b'0'..=b'9' => {
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                out.push(Token {
                    kind: TokenKind::Int,
                    text: &src[start..i],
                    start,
                    end: i,
                });
                continue;
            }
            _ if is_word_start(src, i) => {
                i = scan_word(src, i);
                out.push(Token {
                    kind: TokenKind::Word,
                    text: &src[start..i],
                    start,
                    end: i,
                });
                continue;
            }
            _ => TokenKind::Other,
        };
        // Single-byte tokens (and Other: consume one char, which may be
        // multi-byte UTF-8).
        let ch_len = src[i..].chars().next().map_or(1, char::len_utf8);
        i += ch_len;
        out.push(Token {
            kind,
            text: &src[start..i],
            start,
            end: i,
        });
    }
    out
}

fn is_word_start(src: &str, i: usize) -> bool {
    src[i..].chars().next().is_some_and(|c| c.is_alphabetic())
}

/// Scan a word: letters plus internal `.`/`'`/`-`/`/` (covers `C.L.R.`,
/// `O'Brien`, `S157/2002`-style designators after the leading letter).
fn scan_word(src: &str, start: usize) -> usize {
    let mut end = start;
    for (off, ch) in src[start..].char_indices() {
        let pos = start + off;
        let ok = ch.is_alphabetic()
            || ch.is_ascii_digit()
            || ch == '.'
            || ch == '\''
            || ch == '-'
            || ch == '/';
        if !ok {
            return pos;
        }
        end = pos + ch.len_utf8();
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        lex(src).iter().map(|t| t.kind).collect()
    }

    #[test]
    fn lexes_reported_citation_shape() {
        let toks = lex("Mabo v Queensland (No 2) (1992) 175 CLR 1");
        let texts: Vec<&str> = toks.iter().map(|t| t.text).collect();
        assert_eq!(
            texts,
            vec![
                "Mabo",
                "v",
                "Queensland",
                "(",
                "No",
                "2",
                ")",
                "(",
                "1992",
                ")",
                "175",
                "CLR",
                "1"
            ]
        );
    }

    #[test]
    fn dotted_abbreviation_is_one_word() {
        let toks = lex("175 C.L.R. 1");
        assert_eq!(toks[1].text, "C.L.R.");
        assert_eq!(toks[1].kind, TokenKind::Word);
    }

    #[test]
    fn ranges_are_byte_accurate() {
        let src = "ab (12)";
        let toks = lex(src);
        for t in &toks {
            assert_eq!(&src[t.start..t.end], t.text);
        }
        assert_eq!(
            kinds(src),
            vec![
                TokenKind::Word,
                TokenKind::OpenParen,
                TokenKind::Int,
                TokenKind::CloseParen
            ]
        );
    }

    #[test]
    fn non_ascii_does_not_panic() {
        // Property guarded harder in parser tests; here: no panic, sane kinds.
        let toks = lex("Mabo — Queensland £ (1992)");
        assert!(toks.iter().any(|t| t.kind == TokenKind::Other));
    }

    #[test]
    fn empty_input_lexes_to_nothing() {
        assert!(lex("").is_empty());
        assert!(lex("   \t ").is_empty());
    }
}
