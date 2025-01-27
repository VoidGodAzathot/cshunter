#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub _type: TokenType,
    pub value: Option<String>
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    StructName,
    CloseBracket,
    OpenBracket,
    OpenQuote,
    CloseQuote,
    NameField,
    ValueField
}