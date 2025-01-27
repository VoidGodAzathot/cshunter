use std::{any::type_name, collections::HashMap};

use super::{token::TokenType, tree::Tree};

pub trait Convertable {
    fn generate(map: HashMap<String, HashMap<String, String>>) -> Self;
}

pub fn convert_from_login_user<T: Convertable>(tree: Tree) -> Option<T> {
    let struct_name = type_name::<T>().split("::").last().unwrap();
    let original_struct_name = tree.find_original_struct_name()?;

    if !struct_name.eq(&original_struct_name) && !struct_name.eq("_Unknown_") {
        return None;
    }

    let tokens = tree.tokens;
    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();

    let mut stack: Vec<(String, HashMap<String, String>)> = Vec::new();
    let mut current_name = String::new();
    let mut current_fields = HashMap::new();
    let mut nesting_level = 0;
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i]._type {
            TokenType::StructName => {
                current_name = tokens[i].value.clone().unwrap_or_default();
                i += 1;
            }
            TokenType::OpenBracket => {
                nesting_level += 1;
                if nesting_level > 1 {
                    stack.push((current_name.clone(), current_fields.clone()));
                }
                i += 1;
            }
            TokenType::CloseBracket => {
                if nesting_level > 0 {
                    map.insert(current_name.clone(), current_fields.clone());
                    nesting_level -= 1;
                }
                i += 1;
            }
            TokenType::NameField => {
                if let Some(field_name) = &tokens[i].value {
                    let mut value = None;
                    i += 1; // переходим к следующему токену после NameField

                    // ищем ValueField или StructName до следующего NameField/CloseBracket
                    while i < tokens.len()
                        && !matches!(
                            tokens[i]._type,
                            TokenType::NameField | TokenType::CloseBracket
                        )
                    {
                        match tokens[i]._type {
                            TokenType::ValueField | TokenType::StructName => {
                                value = tokens[i].value.clone();
                                i += 1;
                                break;
                            }
                            _ => i += 1,
                        }
                    }
                    current_fields.insert(field_name.clone(), value.unwrap_or_default());
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    // добавляем последнюю структуру если осталась
    if !current_name.is_empty() {
        map.insert(current_name, current_fields);
    }

    Some(T::generate(map))
}
