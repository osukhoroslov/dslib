use std::fmt::{Error, Formatter};
use std::fs;
use std::rc::Rc;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyTuple};
use serde::Serialize;

use crate::node::{Node, Context};
use crate::system::Message;


#[derive(Clone)]
pub struct JsonMessage {
    pub tip: String,
    pub data: String,
}

impl JsonMessage {
    pub fn new(tip: &str, data: &str) -> Self {
        JsonMessage {
            tip: tip.to_string(),
            data: data.to_string(),
        }
    }

    pub fn from<T>(tip: &str, data: &T) -> Self
    where
        T: ?Sized + Serialize,
    {
        JsonMessage {
            tip: tip.to_string(),
            // TODO: use regex
            data: serde_json::to_string_pretty(data).unwrap().replace("\n", "").replace("  ", ""),
        }
    }
}

impl std::fmt::Debug for JsonMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} {}", self.tip, self.data)
    }
}

impl Message for JsonMessage {
    fn size(&self) -> u64 {
        self.data.len() as u64
    }
}

pub struct PyNodeFactory {
    node_class: PyObject,
    msg_class: Rc<PyObject>,
    ctx_class: Rc<PyObject>,
}

impl PyNodeFactory {
    pub fn new(node_path: &str, node_class: &str) -> Self {
        let node_code = fs::read_to_string(node_path).unwrap();
        let node_realpath = fs::canonicalize(node_path).unwrap();
        let node_filename = node_realpath.to_str().unwrap();
        let node_module = node_filename.replace(".py", "");
        let classes = Python::with_gil(|py| -> (PyObject, PyObject, PyObject) {
            let node_module = PyModule::from_code(
                py, node_code.as_str(), node_filename, &node_module).unwrap();
            let node_class = node_module.getattr(node_class).unwrap().to_object(py);
            let msg_class = node_module.getattr("Message").unwrap().to_object(py);
            let ctx_class = node_module.getattr("Context").unwrap().to_object(py);
            (node_class, msg_class, ctx_class)
        });
        Self {
            node_class: classes.0,
            msg_class: Rc::new(classes.1),
            ctx_class: Rc::new(classes.2),
        }
    }

    pub fn build(&self, node_id: &str, args: impl IntoPy<Py<PyTuple>>, seed: u64) -> PyNode {
        let node = Python::with_gil(|py| -> PyObject {
            py.run(format!("import random\nrandom.seed({})", seed).as_str(), None, None).unwrap();
            self.node_class.call1(py, args)
                .map_err(|e| log_python_error(e, py))
                .unwrap().to_object(py)
        });
        PyNode {
            id: node_id.to_string(),
            node,
            msg_class: self.msg_class.clone(),
            ctx_class: self.ctx_class.clone(),
        }
    }
}

pub struct PyNode {
    id: String,
    node: PyObject,
    msg_class: Rc<PyObject>,
    ctx_class: Rc<PyObject>,
}

impl PyNode {
    fn handle_node_actions(ctx: &mut Context<JsonMessage>, py_ctx: &PyObject, py: Python) {
        let sent: Vec<(String, String, String)> = py_ctx.getattr(py, "_sent_messages").unwrap().extract(py).unwrap();
        for m in sent {
            ctx.send(JsonMessage::new(&m.0, &m.1), &m.2);
        }
        let sent_local: Vec<(String, String)> = py_ctx.getattr(py, "_sent_local_messages").unwrap().extract(py).unwrap();
        for m in sent_local {
            ctx.send_local(JsonMessage::new(&m.0, &m.1));
        }
        let timer_actions: Vec<(String, f64)> = py_ctx.getattr(py, "_timer_actions").unwrap().extract(py).unwrap();
        for t in timer_actions {
            if t.1 < 0.0 {
                ctx.cancel_timer(&t.0);    
            } else {
                ctx.set_timer(&t.0, t.1);
            }
        }
    }
}

impl Node<JsonMessage> for PyNode {
    fn id(&self) -> &String {
        &self.id
    }

    fn on_message(&mut self, msg: JsonMessage, from: String, ctx: &mut Context<JsonMessage>) {
        Python::with_gil(|py| {
            let py_msg = self.msg_class.call_method1(py, "from_json", (msg.tip, msg.data)).unwrap();
            let py_ctx = self.ctx_class.call1(py, (ctx.time(),)).unwrap();
            self.node
                .call_method1(py, "on_message", (py_msg, from, &py_ctx))
                .map_err(|e| log_python_error(e, py))
                .unwrap();
            PyNode::handle_node_actions(ctx, &py_ctx, py);
        });
    }

    fn on_local_message(&mut self, msg: JsonMessage, ctx: &mut Context<JsonMessage>) {
        Python::with_gil(|py| {
            let py_msg = self.msg_class.call_method1(py, "from_json", (msg.tip, msg.data)).unwrap();
            let py_ctx = self.ctx_class.call1(py, (ctx.time(),)).unwrap();
            self.node
                .call_method1(py, "on_local_message", (py_msg, &py_ctx))
                .map_err(|e| log_python_error(e, py))
                .unwrap();
            PyNode::handle_node_actions(ctx, &py_ctx, py);
        });
    }

    fn on_timer(&mut self, timer: String, ctx: &mut Context<JsonMessage>) {
        Python::with_gil(|py| {
            let py_ctx = self.ctx_class.call1(py, (ctx.time(),)).unwrap();
            self.node
                .call_method1(py, "on_timer", (timer, &py_ctx))
                .map_err(|e| log_python_error(e, py))
                .unwrap();
            PyNode::handle_node_actions(ctx, &py_ctx, py);
        });
    }
}

fn log_python_error(e: PyErr, py: Python) -> PyErr {
    eprintln!("\n!!! Error when calling Python code:\n");
    e.print(py);
    eprintln!();
    e
}
