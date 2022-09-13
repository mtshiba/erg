use std::iter::Iterator;

use erg_common::config::Input;

// use erg_compiler::parser;

use erg_parser::error::ParseResult;
use erg_parser::lex::Lexer;
use erg_parser::token::*;
use TokenKind::*;

const FILE1: &str = "tests/test1_basic_syntax.er";
const FILE2: &str = "tests/test2_advanced_syntax.er";

#[test]
fn test_lexer_for_basic() -> ParseResult<()> {
    let mut lexer = Lexer::new(Input::File(FILE1.into()));
    let newline = "\n";
    let /*mut*/ token_array = vec![
        (Newline, newline),
        (Newline, newline),
        (Newline, newline),
        (Symbol, "_a"),
        (Equal, "="),
        (NatLit, "1_234"),
        (Plus, "+"),
        (RatioLit, "1113."),
        (Star, "*"),
        (RatioLit, "3_000.2e-4"),
        (Pow, "**"),
        (NatLit, "0003"),
        (Star, "*"),
        (RatioLit, ".4"),
        (Newline, newline),
        (Symbol, "a"),
        (Comma, ","),
        (UBar, "_"),
        (Comma, ","),
        (Spread, "..."), // EllipsisLit
        (Symbol, "b"),
        (Equal, "="),
        (Symbol, "five_elem_tuple"),
        (Newline, newline),
        (Symbol, "f"),
        (Symbol, "x"),
        (Comma, ","),
        (Symbol, "y"),
        (Equal, "="),
        (Newline, newline),
        (Indent, "    "),
        (Symbol, "x"),
        (Plus, "+"),
        (Symbol, "y"),
        (Newline, newline),
        (Dedent, ""),
        (Symbol, "if!"),
        (BoolLit, "True"),
        (Comma, ","),
        (Symbol, "do!"),
        (Newline, newline),
        (Indent, "    "),
        (Symbol, "print!"),
        (StrLit, "\"\\\\hello, world\\\"\""),
        (Newline, newline),
        (NatLit, "10"),
        (Dot, "."),
        (Symbol, "times!"),
        (Symbol, "do!"),
        (Newline, newline),
        (Indent, "    "),
        (Symbol, "if!"),
        (Symbol, "x"),
        (Dot, "."),
        (Symbol, "y"),
        (Dot, "."),
        (Symbol, "z"),
        (Comma, ","),
        (Symbol, "do!"),
        (Newline, newline),
        (Indent, "    "),
        (Symbol, "print!"),
        (StrLit, "\"\""),
        (Newline, newline),
        (Dedent, ""),
        (Newline, newline),
        (Newline, newline),
        (Symbol, "Hello"),
        (Equal, "="),
        (Symbol, "S2c"),
        (StrLit, "\"hello\""),
        (Newline, newline),
        (Dedent, ""),
        (Dedent, ""),
        (Symbol, "aあ아"),
        (Equal, "="),
        (Newline, newline),
        (Indent, "    "),
        (Newline, newline),
        (StrLit, "\"aaa\""),
        (Newline, newline),
        (Dedent, ""),
        (Symbol, "x"),
        (Semi, ";"),
        (Symbol, "x"),
        (Semi, ";"),
        (Semi, ";"),
        (Symbol, "x"),
        (Semi, ";"),
        (Newline, newline),
        (NatLit, "10"),
        (Closed, ".."),
        (Symbol, "twelve"),
        (Semi, ";"),
        (Newline, newline),
        (EOF, ""),
    ];
    let mut tok: Token;
    for (id, i) in token_array.into_iter().enumerate() {
        tok = lexer.next().unwrap().unwrap();
        assert_eq!(tok, Token::from_str(i.0, i.1));
        print!("{id:>03}: ");
        println!("{tok}");
    }
    Ok(())
}

#[test]
fn test_lexer_for_advanced() -> ParseResult<()> {
    let mut lexer = Lexer::new(Input::File(FILE2.into()));
    let newline = "\n";
    let /*mut*/ token_array = vec![
        (Newline, newline),
        (Newline, newline),
        (Newline, newline),
        (Newline, newline),
        (Symbol, "f"),
        (Symbol, "x"),
        (Equal, "="),
        (NatLit, "1"),
        (Plus, "+"),
        (Symbol, "x"),
        (Plus, "+"),
        (NatLit, "2"),
        (Newline, newline),
        (Symbol, "f"),
        (Symbol, "x"),
        (Comma, ","),
        (Symbol, "y"),
        (Equal, "="),
        (Newline, newline),
        (Indent, "    "),
        (NatLit, "1"),
        (Plus, "+"),
        (Symbol, "x"),
        (Plus, "+"),
        (Symbol, "y"),
        (Newline, newline),
        (Dedent, ""),
        (Symbol, "f"),
        (Symbol, "x"),
        (Comma, ","),
        (Symbol, "y"),
        (Comma, ","),
        (Symbol, "z"),
        (Equal, "="),
        (Newline, newline),
        (Indent, "    "),
        (NatLit, "1"),
        (Plus, "+"),
        (Symbol, "x"),
        (Plus, "+"),
        (Symbol, "y"),
        (Plus, "+"),
        (Symbol, "z"),
        (Newline, newline),
        (Dedent, ""),
        (Symbol, "assert"),
        (NatLit, "4"),
        (DblEq, "=="),
        (Symbol, "f"),
        (NatLit, "1"),
        (Newline, newline),
        (Symbol, "assert"),
        (NatLit, "4"),
        (DblEq, "=="),
        (Symbol, "f"),
        (NatLit, "1"),
        (Comma, ","),
        (NatLit, "1"),
        (Newline, newline),
        (Symbol, "assert"),
        (NatLit, "3"),
        (DblEq, "=="),
        (Symbol, "f"),
        (NatLit, "1"),
        (Comma, ","),
        (NatLit, "1"),
        (Comma, ","),
        (NatLit, "1"),
        (Newline, newline),
        (Newline, newline),
        (Newline, newline),
        (Symbol, "fib"),
        (NatLit, "0"),
        (Equal, "="),
        (NatLit, "0"),
        (Newline, newline),
        (Symbol, "fib"),
        (NatLit, "1"),
        (Equal, "="),
        (NatLit, "1"),
        (Newline, newline),
        (Symbol, "fib"),
        (LParen, "("),
        (Symbol, "n"),
        (Colon, ":"),
        (Symbol, "Nat"),
        (RParen, ")"),
        (FuncArrow, "->"),
        (Symbol, "Nat"),
        (Equal, "="),
        (Symbol, "fib"),
        (LParen, "("),
        (Symbol, "n"),
        (Minus, "-"),
        (NatLit, "1"),
        (RParen, ")"),
        (Plus, "+"),
        (Symbol, "fib"),
        (LParen, "("),
        (Symbol, "n"),
        (Minus, "-"),
        (NatLit, "2"),
        (RParen, ")"),
        (Newline, newline),
        (Newline, newline),
        (Newline, newline),
        (Symbol, "t"),
        (Equal, "="),
        (Symbol, "if"),
        (BoolLit, "True"),
        (Colon, ":"),
        (Newline, newline),
        (Indent, "    "),
        (Symbol, "then"),
        (Colon, ":"),
        (NatLit, "1"),
        (Newline, newline),
        (Symbol, "else"),
        (Colon, ":"),
        (NatLit, "2"),
        (Newline, newline),
        (Dedent, ""),
        (Symbol, "assert"),
        (Symbol, "t"),
        (DblEq, "=="),
        (NatLit, "1"),
        (Newline, newline),
        (Newline, newline),
        (Newline, newline),
        (Symbol, "math"),
        (Equal, "="),
        (Symbol, "import"),
        (StrLit, "\"math\""),
        (Newline, newline),
        (Newline, newline),
        (LBrace, "{"),
        (Symbol, "pi"),
        (RBrace, "}"),
        (Equal, "="),
        (Symbol, "import"),
        (StrLit, "\"math\""),
        (Newline, newline),
        (EOF, ""),
    ];

    let mut tok: Token;
    for (id, i) in token_array.into_iter().enumerate() {
        print!("{id:>03}: ");
        tok = lexer.next().unwrap().unwrap();
        assert_eq!(tok, Token::from_str(i.0, i.1));
        println!("{tok}");
    }
    Ok(())
}

#[test]
fn tesop_te_prec() {
    assert_eq!(Mod.precedence(), Some(170));
    assert_eq!(LParen.precedence(), Some(0));
    assert_eq!(Illegal.precedence(), None);
}
