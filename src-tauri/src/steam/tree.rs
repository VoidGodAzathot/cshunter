use super::token::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct Tree {
    original: String,
    pub tokens: Vec<Token>,
}

impl Tree {
    pub fn new(data: String) -> Self {
        Self {
            original: data,
            tokens: vec![],
        }
    }

    pub fn find_original_struct_name(&self) -> Option<String> {
        let tokens = self.tokens.clone();

        for i in 0..tokens.len() {
            let token = &tokens[i];

            if token._type.eq(&TokenType::OpenQuote) {
                let next_token = &tokens[i + 1];

                if (next_token._type.eq(&TokenType::StructName)).to_owned() {
                    return next_token.value.clone();
                }
            }
        }

        None
    }

    pub fn parse(&mut self) {
        let split_tree_str = self.original.split('\n').collect::<Vec<&str>>();

        for i in 0..split_tree_str.len() {
            let raw_token = split_tree_str[i].trim().to_string();

            if raw_token.eq("{") {
                self.tokens.push(Token {
                    _type: TokenType::OpenBracket,
                    value: None,
                });

                continue;
            }

            if raw_token.eq("}") {
                self.tokens.push(Token {
                    _type: TokenType::CloseBracket,
                    value: None,
                });

                continue;
            }

            if raw_token.starts_with("\"") {
                if raw_token.chars().filter(|c| *c == '"').count() == 2 {
                    // начало структуры
                    self.tokens.push(Token {
                        _type: TokenType::OpenQuote,
                        value: None,
                    });
                    self.tokens.push(Token {
                        _type: TokenType::StructName,
                        value: Some(raw_token.replace("\"", "")),
                    });
                    self.tokens.push(Token {
                        _type: TokenType::CloseQuote,
                        value: None,
                    });

                    continue;
                } else {
                    // парсим поле
                    let to = raw_token.split("\t\t").collect::<Vec<&str>>();

                    self.tokens.push(Token {
                        _type: TokenType::OpenQuote,
                        value: None,
                    });
                    self.tokens.push(Token {
                        _type: TokenType::NameField,
                        value: Some(to[0].replace("\"", "")),
                    });
                    self.tokens.push(Token {
                        _type: TokenType::CloseQuote,
                        value: None,
                    });

                    self.tokens.push(Token {
                        _type: TokenType::OpenQuote,
                        value: None,
                    });
                    self.tokens.push(Token {
                        _type: TokenType::ValueField,
                        value: Some(to[1].replace("\"", "")),
                    });
                    self.tokens.push(Token {
                        _type: TokenType::CloseQuote,
                        value: None,
                    });
                }
            }
        }
    }
}
