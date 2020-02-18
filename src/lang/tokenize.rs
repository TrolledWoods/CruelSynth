use std::iter::Peekable;
use crate::operator::Operator;

#[derive(Debug)]
pub struct TokenError {
    pub kind: TokenErrorKind,
    pub pos: (usize, usize),
}

#[derive(Debug)]
pub enum TokenErrorKind {
    ExpectedOperator,
    UnexpectedToken,
    EmptyIdentifier,
    InvalidFloat,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub pos: (usize, usize),
    pub kind: TokenKind
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Operator(Operator),
    Assignment,
    Separator(char),
    Identifier(String),
    Float(f32),
    Block(char, Vec<Token>),
}

pub fn tokenize(code: &str) -> Result<Vec<Token>, TokenError> {
    tokenize_setup(&mut code.chars().peekable(), &mut (0, 0), None)
}

fn tokenize_setup(code: &mut Peekable<impl Iterator<Item = char>>, 
                  pos: &mut (usize, usize), 
                  terminator: Option<char>) -> Result<Vec<Token>, TokenError> {
    let mut tokens = Vec::new();

    loop {
        skip_whitespace(code, pos);

        if let Some(&c) = code.peek() {
            match c {
                '(' => {
                    let token_pos = pos.clone();
                    pos.1 += 1;
                    code.next();
                    let block_tokens = tokenize_setup(code, pos, Some(')'))?;
                    tokens.push(Token {
                        kind: TokenKind::Block('(', block_tokens),
                        pos: token_pos
                    });
                },
                '[' => {
                    let token_pos = pos.clone();
                    pos.1 += 1;
                    code.next();
                    let block_tokens = tokenize_setup(code, pos, Some(']'))?;
                    tokens.push(Token {
                        kind: TokenKind::Block('[', block_tokens),
                        pos: token_pos
                    });
                },
                c if c.is_digit(10) || c == '.' => {
                    let float_pos = pos.clone();
                    let float = read_float(code, pos)?;
                    tokens.push(Token {
                        kind: TokenKind::Float(float),
                        pos: float_pos
                    });
                },
                c if c.is_alphabetic() || c == '_' => {
                    let token_pos = pos.clone();
                    let identifier = read_identifier(code, pos)?;
                    tokens.push(Token {
                        kind: TokenKind::Identifier(identifier),
                        pos: token_pos
                    });
                },
                _ => {
                    return Err(TokenError {
                        kind: TokenErrorKind::UnexpectedToken,
                        pos: pos.clone()
                    });
                },
            }
        }
    }
}

fn read_float(code: &mut Peekable<impl Iterator<Item = char>>, pos: &mut (usize, usize))
        -> Result<f32, TokenError> {
    let float_pos = pos.clone();
    let mut temp_string = String::new();
    let mut contains_dot = false;
    while let Some(&c) = code.peek() {
        if c.is_digit(10){
            temp_string.push(c);
            code.next();
            pos.1 += 1;
        }else if c == '.' && !contains_dot {
            contains_dot = true;
            temp_string.push(c);
            code.next();
            pos.1 += 1;
        }else{
            break;
        }
    }

    let float: Result<f32, _> = temp_string.parse();
    if let Ok(float) = float {
        Ok(float)
    }else{
        Err(TokenError {
            kind: TokenErrorKind::InvalidFloat,
            pos: float_pos
        })
    }
}

fn read_identifier(code: &mut Peekable<impl Iterator<Item = char>>, pos: &mut (usize, usize))
    -> Result<String, TokenError> {
    let identifier_pos = pos.clone();
    let mut identifier = String::new();
    while let Some(&c) = code.peek() {
        if c.is_digit(10) || c.is_alphabetic() || c == '_' {
            code.next();
            pos.1 += 1;
            identifier.push(c);
        }else{
            break;
        }
    }

    if identifier.len() == 0 {
        return Err(TokenError {
            kind: TokenErrorKind::EmptyIdentifier,
            pos: identifier_pos
        });
    }

    Ok(identifier)
}

fn skip_whitespace(code: &mut Peekable<impl Iterator<Item = char>>, pos: &mut (usize, usize)) {
    while let Some(&value) = code.peek() {
        if value.is_whitespace() {
            if value == '\n' {
                pos.1 = 0;
                pos.0 += 1;
            }else{
                pos.1 += 1;
            }

            code.next();
        }else{
            break;
        }
    }
}
