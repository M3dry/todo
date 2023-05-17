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
        TextOp::Verbatim(ops) => format!("(box :class \"verbatim-text\" :halign \"start\" {})", ops.into_iter().map(|op| op_to_string(op)).collect::<Vec<String>>().join("")),
        TextOp::Underline(ops) => format!("(box :class \"underline-text\" :halign \"start\" {})", ops.into_iter().map(|op| op_to_string(op)).collect::<Vec<String>>().join("")),
        TextOp::Crossed(ops) => format!("(box :class \"crossed-text\" :halign \"start\" {})", ops.into_iter().map(|op| op_to_string(op)).collect::<Vec<String>>().join("")),
        TextOp::Bold(ops) => format!("(box :class \"bold-text\" :halign \"start\" {})", ops.into_iter().map(|op| op_to_string(op)).collect::<Vec<String>>().join("")),
        TextOp::Italic(ops) => format!("(box :class \"italic-text\" :halign \"start\" {})", ops.into_iter().map(|op| op_to_string(op)).collect::<Vec<String>>().join("")),
        TextOp::Normal(str) => format!("(label :class \"normal-text\" :halign \"start\" :text \"{str}\")"),
    }
}
