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
                description: todo
                    .description
                    .0
                    .iter()
                    .map(|op| op_to_string(op))
                    .collect::<Vec<String>>(),
            })
            .collect()
    }
}

fn op_to_string(op: &TextOp) -> String {
    match op {
        TextOp::Verbatim(ops) => format!(
            "(box :style \"color: #c3e88d;\" :halign \"start\" {})",
            ops.into_iter()
                .map(|op| op_to_string(op))
                .collect::<Vec<String>>()
                .join("")
        ),
        TextOp::Underline(ops) => format!(
            "(box :style \"text-decoration: underline;\" :halign \"start\" {})",
            ops.into_iter()
                .map(|op| op_to_string(op))
                .collect::<Vec<String>>()
                .join("")
        ),
        TextOp::Crossed(ops) => format!(
            "(box :style \"text-decoration: line-through;\" :halign \"start\" {})",
            ops.into_iter()
                .map(|op| op_to_string(op))
                .collect::<Vec<String>>()
                .join("")
        ),
        TextOp::Bold(ops) => format!(
            "(box :style \"font-weight: bold;\" :halign \"start\" {})",
            ops.into_iter()
                .map(|op| op_to_string(op))
                .collect::<Vec<String>>()
                .join("")
        ),
        TextOp::Italic(ops) => format!(
            "(box :style \"font-style: italic;\" :halign \"start\" {})",
            ops.into_iter()
                .map(|op| op_to_string(op))
                .collect::<Vec<String>>()
                .join("")
        ),
        TextOp::Link { name, handler, path } => todo!(),
        TextOp::TextExtra(char, ops) => format!(
            "(box :space-evenly false :halign \"start\" (label :halign \"start\" :text \"{char}\") {})",
            ops.into_iter()
                .map(|op| op_to_string(op))
                .collect::<Vec<String>>()
                .join("")
        ),
        TextOp::Normal(str) => format!("(label :halign \"start\" :text \"{str}\")"),
    }
}
