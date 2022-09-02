use std::fmt::{Error, Formatter};
use std::fs;
use std::rc::Rc;
use lazy_static::lazy_static;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyTuple};
use regex::Regex;
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

    fn corrupt(&mut self) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#""\w+""#).unwrap();
        }
        self.data = RE.replace_all(&*self.data, "\"\"").to_string();
    }
}

pub struct PyNodeFactory {
    node_class: PyObject,
    msg_class: Rc<PyObject>,
    ctx_class: Rc<PyObject>,
    get_size_fun: Rc<Py<PyAny>>,
}

impl PyNodeFactory {
    pub fn new(node_path: &str, node_class: &str) -> Self {
        let node_code = fs::read_to_string(node_path).unwrap();
        let node_realpath = fs::canonicalize(node_path).unwrap();
        let node_filename = node_realpath.to_str().unwrap();
        let node_module = node_filename.replace(".py", "");
        let classes = Python::with_gil(|py| -> (PyObject, PyObject, PyObject, Py<PyAny>) {
            let node_module = PyModule::from_code(
                py, node_code.as_str(), node_filename, &node_module).unwrap();
            let node_class = node_module.getattr(node_class).unwrap().to_object(py);
            let msg_class = node_module.getattr("Message").unwrap().to_object(py);
            let ctx_class = node_module.getattr("Context").unwrap().to_object(py);
            let get_size_fun = get_size_fun(py);
            (node_class, msg_class, ctx_class, get_size_fun)
        });
        Self {
            node_class: classes.0,
            msg_class: Rc::new(classes.1),
            ctx_class: Rc::new(classes.2),
            get_size_fun: Rc::new(classes.3),
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
            get_size_fun: self.get_size_fun.clone(),
            max_size: 0,
            max_size_freq: 0,
            max_size_counter: 0,
        }
    }
}

pub struct PyNode {
    id: String,
    node: PyObject,
    msg_class: Rc<PyObject>,
    ctx_class: Rc<PyObject>,
    get_size_fun: Rc<Py<PyAny>>,
    max_size: u64,
    max_size_freq: u32,
    max_size_counter: u32,
}

impl PyNode {
    pub fn set_max_size_freq(&mut self, freq: u32) {
        self.max_size_freq = freq;
        self.max_size_counter = 1;
    }

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

    fn update_max_size(&mut self, py: Python, force_update: bool) {
        if self.max_size_freq > 0 {
            self.max_size_counter -= 1;
            if self.max_size_counter == 0 || force_update {
                let size: u64 = self.get_size_fun.call1(py, (&self.node,)).unwrap().extract(py).unwrap();
                // let size: u64 = self.node.call_method0(py, "get_size").unwrap().extract(py).unwrap();
                self.max_size = self.max_size.max(size);
                self.max_size_counter = self.max_size_freq;
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
            self.update_max_size(py, false);
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
            self.update_max_size(py, false);
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
            self.update_max_size(py, false);
        });
    }

    fn max_size(&mut self) -> u64 {
        Python::with_gil(|py| {
            self.update_max_size(py, true)
        });
        self.max_size
    }
}

fn log_python_error(e: PyErr, py: Python) -> PyErr {
    eprintln!("\n!!! Error when calling Python code:\n");
    e.print(py);
    eprintln!();
    e
}

fn get_size_fun(py: Python) -> Py<PyAny> {
    PyModule::from_code(
        py,
        "
import sys

def get_size(obj, seen=None):
    size = sys.getsizeof(obj)
    if seen is None:
        seen = set()
    obj_id = id(obj)
    if obj_id in seen:
        return 0
    seen.add(obj_id)
    if isinstance(obj, dict):
        size += sum([get_size(v, seen) for v in obj.values()])
        size += sum([get_size(k, seen) for k in obj.keys()])
    elif hasattr(obj, '__dict__'):
        size += get_size(obj.__dict__, seen)
    elif hasattr(obj, '__iter__') and not isinstance(obj, (str, bytes, bytearray)):
        size += sum([get_size(i, seen) for i in obj])
    return size",
        "",
        "",
    ).unwrap().getattr("get_size").unwrap().into()
}
