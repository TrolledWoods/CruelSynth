use std::iter::Peekable;
use crate::operator::Operator;
use std::fmt::{ self, Display, Debug, Formatter };

#[derive(Debug)]
pub struct TokenError {
    pub kind: TokenErrorKind,
    pub pos: (usize, usize),
}

#[derive(Debug)]
pub enum TokenErrorKind {
    ExpectedToken(char),
    UnexpectedEndOfFile,
    UnexpectedToken(char),
    EmptyIdentifier,
    InvalidFloat,
    InvalidOperator,
}

#[derive(PartialEq)]
pub struct Token {
    pub pos: (usize, usize),
    pub kind: TokenKind
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.kind)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.kind)
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Operator(Operator),
    Assignment,
    Separator(char),
    Identifier(String),
    Float(f32),
    Block(BlockKind, Vec<Token>),
    Variable,
    CommandTerminator,
}

/// Ignore this type, it is just because
/// the trait system is a bit annoying
/// sometimes
pub struct TokenSlice<'a>(&'a [Token]);

impl Display for TokenSlice<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, token) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " {}", token)?;
            }else{
                write!(f, "{}", token)?;
            }
        }

        Ok(())
    }
}

pub fn as_displayable_tokens<'a>(tokens: &'a [Token]) -> TokenSlice<'a> {
    TokenSlice(tokens)
}

pub fn print_tokens(tokens: &[Token]) {
    println!("{}", as_displayable_tokens(tokens));
}

#[derive(Debug, PartialEq)]
pub enum BlockKind {
    Parenthesis,
    Bracket
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use TokenKind::*;
        match self {
            Variable => write!(f, "$")?,
            Operator(op) => write!(f, "{:?}", op)?,
            Assignment => write!(f, ":")?,
            Separator(c) => write!(f, "{}", c)?,
            Identifier(s) => write!(f, "{}", s)?,
            Float(float) => write!(f, "{}", float)?,
            Block(kind, contents) => {
                write!(f, "{}", match kind {
                    BlockKind::Parenthesis => '(',
                    BlockKind::Bracket => '['
                })?;

                write!(f, "{}", as_displayable_tokens(&contents[..]))?;

                write!(f, "{}", match kind {
                    BlockKind::Parenthesis => ')',
                    BlockKind::Bracket => ']'
                })?;
            },
            CommandTerminator => write!(f, ";")?,
        }

        Ok(())
    }
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
            // See if we should terminate the tokenizing
            if let Some(terminator) = terminator {
                if c == terminator {
                    pos.1 += 1;
                    code.next();
                    break;
                }
            }

            // See if it's the beginning of a token
            match c {
                '(' => {
                    let token_pos = pos.clone();
                    pos.1 += 1;
                    code.next();
                    let block_tokens = tokenize_setup(code, pos, Some(')'))?;
                    tokens.push(Token {
                        kind: TokenKind::Block(BlockKind::Parenthesis, block_tokens),
                        pos: token_pos
                    });
                },
                '[' => {
                    let token_pos = pos.clone();
                    pos.1 += 1;
                    code.next();
                    let block_tokens = tokenize_setup(code, pos, Some(']'))?;
                    tokens.push(Token {
                        kind: TokenKind::Block(BlockKind::Bracket, block_tokens),
                        pos: token_pos
                    });
                },
                '$' => {
                    tokens.push(Token {
                        kind: TokenKind::Variable,
                        pos: pos.clone()
                    });
                    pos.1 += 1;
                    code.next();
                },
                ':' => {
                    tokens.push(Token {
                        kind: TokenKind::Assignment,
                        pos: pos.clone()
                    });
                    pos.1 += 1;
                    code.next();
                },
                ',' => {
                    tokens.push(Token {
                        kind: TokenKind::Separator(','),
                        pos: pos.clone()
                    });
                    pos.1 += 1;
                    code.next();
                },
                ';' => {
                    tokens.push(Token {
                        kind: TokenKind::CommandTerminator,
                        pos: pos.clone()
                    });
                    pos.1 += 1;
                    code.next();
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
                    let orig_pos = pos.clone();
                    if let Ok(operator) = read_operator(code, pos) {
                        tokens.push(Token {
                            kind: TokenKind::Operator(operator),
                            pos: orig_pos,
                        });
                    }else{
                        return Err(TokenError {
                            kind: TokenErrorKind::UnexpectedToken(c),
                            pos: orig_pos
                        });
                    }
                },
            }
        }else {
            if let Some(terminator) = terminator {
                return Err(TokenError {
                    kind: TokenErrorKind::ExpectedToken(terminator),
                    pos: pos.clone()
                });
            }else{
                break;
            }
        }
    }

    Ok(tokens)
}

fn read_operator(code: &mut Peekable<impl Iterator<Item = char>>, pos: &mut (usize, usize))
        -> Result<Operator, TokenError> {
    let op_pos = pos.clone();
    if let Some(&c) = code.peek() {
        use Operator::*;
        let operator = match c {
            '+' => Some(Add),
            '-' => Some(Sub),
            '*' => Some(Mult),
            '/' => Some(Div),
            '%' => Some(Mod),
            _ => None,
        };

        if let Some(operator) = operator {
            // Since we read an operator, we should
            // increment the counter.
            // NOTE: If there ever is an
            // operator with 2 symbols, this code
            // will have to change slightly
            code.next();
            pos.1 += 1;

            Ok(operator)
        }else{
            Err(TokenError {
                kind: TokenErrorKind::InvalidOperator,
                pos: op_pos,
            })
        }
    }else{
        Err(TokenError {
            kind: TokenErrorKind::UnexpectedEndOfFile,
            pos: op_pos,
        })
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
        if value == '#' {
            while let Some(value) = code.next() {
                if value == '\n' {
                    pos.1 = 0;
                    pos.0 += 1;
                    return;
                }else{
                    pos.1 += 1;
                }
            }
        }else if value.is_whitespace() {
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
