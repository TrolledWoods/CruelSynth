use std::collections::HashMap;
use crate::operator::Operator;
use crate::lang::tokenize::{ Token, TokenKind, BlockKind };
use std::iter::Peekable;

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub pos: Option<(usize, usize)>,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    UnexpectedEndOfFile,
    UnexpectedToken,
    ExpectedFloat,
    ExpectedArgList,
    ExpectedCommandTerminator,
    ExpectedAssignment,
    ExpectedSeparator,
    ExpectedIdentifier,
}

pub struct Node<T> {
    pub kind: T,
    pub pos: Option<(usize, usize)>,
}

impl<T> Node<T> {
    pub fn new(kind: T) -> Node<T> {
        Node {
            kind: kind,
            pos: None
        }
    }

    pub fn with_pos(kind: T, pos: (usize, usize)) -> Node<T> {
        Node {
            kind: kind,
            pos: Some(pos)
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Debug)]
pub enum ExpressionNode {
    Float(f32),
    Variable(String),
    Operator(Operator, Vec<Node<ExpressionNode>>),
    FunctionCall(String, 
                 HashMap<String, 
                 Node<f32>>, 
                 Vec<Node<ExpressionNode>>),
}

#[derive(Debug)]
pub enum CommandNode {
    Assignment(String, Box<Node<ExpressionNode>>),
}

pub fn parse_tokens(tokens: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Vec<Node<CommandNode>>, ParseError> {
    let mut tokens = tokens.peekable();

    let mut commands = Vec::new();
    while let Some(_) = tokens.peek() {
        commands.push(parse_command(&mut tokens)?);
    }

    Ok(commands)
}

// ------------------------ WIERD PARSER CODE -----------------------
// The code here is a bit odd, because I wanted to try
// and implement a tree based parser. The idea is that,
// the "parse_command" function parses a command where 
// no tokens have been found yet. After one has been found,
// it will go into the appropriate sub function, i.e. a branch
// of the tree. the "parse_command__ident" function is a sub function
// where it has previously found an identifier. This way, we
// can produce a recursive tree of parser options with minimal
// overhead. Wether or not this is easy to follow remains
// seen.


fn parse_command(tokens: &mut impl Iterator<Item = Token>) 
        -> Result<Node<CommandNode>, ParseError> {
    let mut tokens = tokens.peekable();
    if let Some(token) = tokens.next() {
        match token.kind {
            TokenKind::Identifier(identifier) => {
                parse_command__ident(&mut tokens, token.pos.clone(), identifier)
            },
            _ => {
                Err(ParseError {
                    kind: ParseErrorKind::UnexpectedToken,
                    pos: Some(token.pos.clone())
                })
            }
        }
    }else{
        Err(ParseError {
            kind: ParseErrorKind::UnexpectedEndOfFile,
            pos: None
        })
    }
}

fn parse_command__ident(tokens: &mut Peekable<impl Iterator<Item = Token>>,
                        pos: (usize, usize), ident: String)
        -> Result<Node<CommandNode>, ParseError> {
    if let Some(token) = tokens.next() {
        match token.kind {
            TokenKind::Assignment => {
                // We now know that we are assigning a variable.
                let expression = parse_expression(tokens)?;

                let next_token = tokens.next();
                match next_token {
                    Some( Token { pos: _, 
                            kind: TokenKind::CommandTerminator }) => {
                        Ok(
                            Node::with_pos(
                                CommandNode::Assignment(
                                    ident, 
                                    Box::new(expression)
                                ),
                                pos
                            )
                        )
                    },
                    _ => {
                        Err(ParseError {
                            kind: ParseErrorKind::ExpectedCommandTerminator,
                            pos: next_token.map(|v| v.pos)
                        })
                    },
                }
            },
            _ => {
                Err(ParseError {
                    kind: ParseErrorKind::UnexpectedToken,
                    pos: Some(token.pos)
                })
            }
        }
    }else{
        Err(ParseError {
            kind: ParseErrorKind::UnexpectedEndOfFile,
            pos: None
        })
    }
}

fn parse_expression(tokens: &mut Peekable<impl Iterator<Item = Token>>)
        -> Result<Node<ExpressionNode>, ParseError> {
    if let Some(token) = tokens.next() {
        match token.kind {
            TokenKind::Float(value) => {
                // It's (just) a float, for sure!
                Ok(Node::with_pos(
                        ExpressionNode::Float(value),
                        token.pos
                    ))
            },
            TokenKind::Variable => {
                // We should have an identifier after this
                match tokens.next() {
                    Some(Token {
                        kind: TokenKind::Identifier(name),
                        pos
                    }) => {
                        Ok(Node::with_pos(
                            ExpressionNode::Variable(name),
                            pos
                        ))
                    },
                    Some(Token { pos, .. }) => Err(
                        ParseError {
                            kind: ParseErrorKind::ExpectedIdentifier,
                            pos: Some(pos)
                        }
                    ),
                    _ => Err(
                        ParseError {
                            kind: ParseErrorKind::UnexpectedEndOfFile,
                            pos: None
                        }
                    )
                }
            },
            TokenKind::Operator(op) => {
                let args = parse_args_list(tokens)?;
                let pos = token.pos;

                Ok(
                    Node::with_pos(
                        ExpressionNode::Operator(op, args),
                        pos
                        )
                )
            },
            TokenKind::Identifier(name) => {
                Ok(parse_function(tokens, name)?)
            },
            _ => {
                println!("hi");
                Err(ParseError {
                kind: ParseErrorKind::UnexpectedToken,
                pos: Some(token.pos)
                })}
        }
    }else{
        Err(ParseError {
            kind: ParseErrorKind::UnexpectedEndOfFile,
            pos: None
        })
    }
}

fn parse_function(tokens: &mut Peekable<impl Iterator<Item = Token>>, name: String)
        -> Result<Node<ExpressionNode>, ParseError> {
    match tokens.peek() {
        Some(
            Token { 
                kind: TokenKind::Block(
                          BlockKind::Bracket,
                          _
                          ), pos }) => {
            // A bracket means that we add some const properties
            // to the function. Later though, we will parse an
            // expression list also
            let pos = *pos;
            // Wierd trickery because we peeked earlier and only got a borrow,
            // not the real thing :/
            let const_args = if let Some(Token{kind:TokenKind::Block(_,const_args),..}) = tokens.next() { 
                parse_const_args_list(&mut const_args.into_iter().peekable())?
            }else{ panic!("hi :=)"); };
            let expressions = parse_args_list(tokens)?;

            Ok(Node::with_pos(
                ExpressionNode::FunctionCall(name, 
                                             const_args, 
                                             expressions),
                pos
            ))
        },
        _ => {
            let expressions = parse_args_list(tokens)?;

            Ok(Node::new(
                ExpressionNode::FunctionCall(name, 
                                HashMap::new(), 
                                expressions)
            ))
        },
    }
}

fn parse_args_list(tokens: &mut Peekable<impl Iterator<Item = Token>>)
        -> Result<Vec<Node<ExpressionNode>>, ParseError> {
    match tokens.peek() {
        Some(Token { kind: TokenKind::Block(BlockKind::Parenthesis, _), .. }) => {
            if let Some(Token { 
                    kind: TokenKind::Block(BlockKind::Parenthesis, contents), 
                    .. }) = tokens.next() {
                Ok(parse_expression_list(&mut contents.into_iter().peekable())?)
            }else{
                panic!("Something isn't right here, the match and the if gave different results....");
            }
        },
        c => {
            Ok(vec![parse_expression(tokens)?])
        }
    }
}

fn parse_const_args_list(tokens: &mut Peekable<impl Iterator<Item = Token>>)
        -> Result<HashMap<String, Node<f32>>, ParseError> {
    if tokens.peek().is_none() {
        return Ok(HashMap::new());
    }

    let mut map = HashMap::new();
    loop {
        let name = match tokens.next() {
            Some(Token { kind: TokenKind::Identifier(name), .. }) => name,
            Some(Token { pos, .. }) => return Err(ParseError {
                    kind: ParseErrorKind::ExpectedIdentifier,
                    pos: Some(pos),
            }),
            _ => return Err(ParseError {
                kind: ParseErrorKind::UnexpectedEndOfFile,
                pos: None
            }),
        };

        match tokens.next() {
            Some(Token { kind: TokenKind::Assignment, .. }) => (),
            Some(Token { pos, .. }) => return Err(ParseError {
                    kind: ParseErrorKind::ExpectedAssignment,
                    pos: Some(pos),
            }),
            _ => return Err(ParseError {
                kind: ParseErrorKind::UnexpectedEndOfFile,
                pos: None,
            }),
        }

        let (pos, value) = match tokens.next() {
            Some(Token { kind: TokenKind::Float(value), pos }) => (pos, value),
            Some(Token { pos, .. }) => return Err(ParseError {
                kind: ParseErrorKind::ExpectedFloat,
                pos: Some(pos),
            }),
            _ => return Err(ParseError {
                kind: ParseErrorKind::UnexpectedEndOfFile,
                pos: None,
            }),
        };
        
        map.insert(name, Node::with_pos(value, pos));

        match tokens.next() {
            Some(Token { kind: TokenKind::Separator(','), .. }) => {
                continue;
            },
            _ => break,
        }
    }

    if let Some(token) = tokens.next() {
        println!("{:?}", &token);
        Err(ParseError {
            kind: ParseErrorKind::ExpectedSeparator,
            pos: Some(token.pos)
        })
    }else{
        Ok(map)
    }
}

fn parse_expression_list(tokens: &mut Peekable<impl Iterator<Item = Token>>)
        -> Result<Vec<Node<ExpressionNode>>, ParseError> {
    // Handle the case of an empty list
    let mut tokens = tokens.peekable();
    if tokens.peek().is_none() {
        return Ok(Vec::new());
    }

    let mut expressions = Vec::new();
    loop {
        expressions.push(parse_expression(&mut tokens)?);

        match tokens.next() {
            Some(Token { kind: TokenKind::Separator(','), .. }) => {
                continue;
            },
            _ => break,
        }
    }

    if let Some(token) = tokens.next() {
        println!("{:?}", &token);
        Err(ParseError {
            kind: ParseErrorKind::ExpectedSeparator,
            pos: Some(token.pos)
        })
    }else{
        Ok(expressions)
    }
}
