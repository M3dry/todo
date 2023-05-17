use serde::{Deserialize, Serialize};

use crate::config::Config;

use super::parser::{Parse, TextOp, Todo};

#[derive(Debug, Serialize, Deserialize)]
pub struct EwwTodo {
    state: String,
    description: Vec<String>,
}

impl EwwTodo {
    pub fn from_todos(todos: Vec<&Todo>, config: &Config) -> Vec<Self> {
        todos
            .into_iter()
            .map(|todo| Self {
                state: todo.state.print(config),
                description: todo.description.0.iter().map(|op| {
                    op_to_string(op)
                }).collect::<Vec<String>>(),
            })
            .collect()
    }
}

fn op_to_string(op: &TextOp) -> String {
    match op {
        TextOp::Verbatim(op) => format!("(box :class \"verbatim-text\" :halign \"start\" {})", op_to_string(op)),
        TextOp::Underline(op) => format!("(box :class \"underline-text\" :halign \"start\" {})", op_to_string(op)),
        TextOp::Crossed(op) => format!("(box :class \"crossed-text\" :halign \"start\" {})", op_to_string(op)),
        TextOp::Bold(op) => format!("(box :class \"bold-text\" :halign \"start\" {})", op_to_string(op)),
        TextOp::Italic(op) => format!("(box :class \"italic-text\" :halign \"start\" {})", op_to_string(op)),
        TextOp::Normal(str) => format!("(label :class \"normal-text\" :halign \"start\" :text \"{str}\")"),
    }
}
