use std::fs;
use pyo3::prelude::*;
use pyo3::types::{PyModule};
use std::io::prelude::*;

use crate::pynode::{JsonMessage, log_python_error};

static LOG_FILE_PATH: &str = "events.log";
static PY_CODE_PATH: &str = "/python/debugger/debugger.py";

#[derive(Debug, Clone)]
pub enum DebugEvent {
    MessageSend {
        msg: JsonMessage,
        src: String,
        dst: String,
        ts: f64
    },
    MessageReceive {
        msg: JsonMessage,
        src: String,
        dst: String,
        ts: f64
    },
    LocalMessageSend {
        msg: JsonMessage,
        dst: String,
        ts: f64
    },
    LocalMessageReceive {
        msg: JsonMessage,
        dst: String,
        ts: f64
    },
    MessageDropped {
        msg: JsonMessage,
        src: String,
        dst: String,
        ts: f64
    },
    MessageDiscarded {
        msg: JsonMessage,
        src: String,
        dst: String,
        ts: f64
    },
    TimerSet {
        name: String,
        delay: f64,
        ts: f64
    },
    TimerFired {
        name: String,
        node: String,
        ts: f64
    },
    // TODO: gather node events? (add field "what happened")
    NodeRecovered {
        node: String,
        ts: f64
    },
    NodeRestarted {
        node: String,
        ts: f64
    },
    NodeCrashed {
        node: String,
        ts: f64
    },
    NodeConnected {
        node: String,
        ts: f64
    },
    NodeDisconnected {
        node: String,
        ts: f64
    },
    LinkEnabled {
        src: String,
        dst: String,
        ts: f64
    },
    LinkDisabled {
        src: String,
        dst: String,
        ts: f64
    },
    NetworkPartition {
        group1: String,  // TODO: change style? (now like: ["1", "2"])
        group2: String,
        ts: f64
    }
}

impl DebugEvent {
    pub fn serialize(&self) -> String {
        match &*self {
            DebugEvent::MessageSend { msg, src, dst, ts } => {
                format!(
                    r#"
                        {{
                            "type": "MessageSend",
                            "data": {{
                                "msg": {{
                                    "type": "{}",
                                    "data": {}
                                }},
                                "src": "{}",
                                "dst": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    msg.tip,
                    msg.data,
                    src,
                    dst,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::MessageReceive { msg, src, dst, ts } => {
                format!(
                    r#"
                        {{
                            "type": "MessageReceive",
                            "data": {{
                                "msg": {{
                                    "type": "{}",
                                    "data": {}
                                }},
                                "src": "{}",
                                "dst": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    msg.tip,
                    msg.data,
                    src,
                    dst,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::LocalMessageSend { msg, dst, ts } => {
                format!(
                    r#"
                        {{
                            "type": "LocalMessageSend",
                            "data": {{
                                "msg": {{
                                    "type": "{}",
                                    "data": {}
                                }},
                                "dst": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    msg.tip,
                    msg.data,
                    dst,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::LocalMessageReceive { msg, dst, ts } => {
                format!(
                    r#"
                        {{
                            "type": "LocalMessageReceive",
                            "data": {{
                                "msg": {{
                                    "type": "{}",
                                    "data": {}
                                }},
                                "dst": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    msg.tip,
                    msg.data,
                    dst,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::MessageDropped { msg, src, dst, ts } => {
                format!(
                    r#"
                        {{
                            "type": "MessageDropped",
                            "data": {{
                                "msg": {{
                                    "type": "{}",
                                    "data": {}
                                }},
                                "src": "{}",
                                "dst": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    msg.tip,
                    msg.data,
                    src,
                    dst,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::MessageDiscarded { msg, src, dst, ts } => {
                format!(
                    r#"
                        {{
                            "type": "MessageDiscarded",
                            "data": {{
                                "msg": {{
                                    "type": "{}",
                                    "data": {}
                                }},
                                "src": "{}",
                                "dst": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    msg.tip,
                    msg.data,
                    src,
                    dst,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::TimerSet { name, delay, ts } => {
                format!(
                    r#"
                        {{
                            "type": "TimerSet",
                            "data": {{
                                "name": "{}",
                                "delay": {},
                                "ts": {}
                            }}
                        }}
                    "#,
                    name,
                    delay,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::TimerFired { name, node, ts } => {
                format!(
                    r#"
                        {{
                            "type": "TimerFired",
                            "data": {{
                                "name": {},
                                "node": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    name,
                    node,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::NodeRecovered { node, ts } => {
                format!(
                    r#"
                        {{
                            "type": "NodeRecovered",
                            "data": {{
                                "node": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    node,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::NodeRestarted { node, ts } => {
                format!(
                    r#"
                        {{
                            "type": "NodeRestarted",
                            "data": {{
                                "node": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    node,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::NodeCrashed { node, ts } => {
                format!(
                    r#"
                        {{
                            "type": "NodeCrashed",
                            "data": {{
                                "node": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    node,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::NodeConnected { node, ts } => {
                format!(
                    r#"
                        {{
                            "type": "NodeConnected",
                            "data": {{
                                "node": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    node,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::NodeDisconnected { node, ts } => {
                format!(
                    r#"
                        {{
                            "type": "NodeDisconnected",
                            "data": {{
                                "node": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    node,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::LinkEnabled { src, dst, ts } => {
                format!(
                    r#"
                        {{
                            "type": "LinkEnabled",
                            "data": {{
                                "src": "{}",
                                "dst": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    src,
                    dst,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::LinkDisabled { src, dst, ts } => {
                format!(
                    r#"
                        {{
                            "type": "LinkDisabled",
                            "data": {{
                                "src": "{}",
                                "dst": "{}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    src,
                    dst,
                    ts
                ).replace("\n", "").replace("  ", "")
            },
            DebugEvent::NetworkPartition { group1, group2, ts } => {
                format!(
                    r#"
                        {{
                            "type": "NetworkPartition",
                            "data": {{
                                "group1": "{:?}",
                                "group2": "{:?}",
                                "ts": {}
                            }}
                        }}
                    "#,
                    group1,
                    group2,
                    ts
                ).replace("\n", "").replace("  ", "")
            }
        }
    }
}

pub fn init_debugger() {
    fs::File::create(LOG_FILE_PATH).unwrap();
}

pub fn add_node_ids(node_ids: &Vec<String>) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(LOG_FILE_PATH)
        .unwrap();
    f.write(b"NODE_IDS").unwrap();
    for id in node_ids.iter() {
        let mut id_with_delim = String::from(":");
        id_with_delim.push_str(id);
        f.write(id_with_delim.as_bytes()).unwrap();
    }
    f.write(b"\n").unwrap();
}

pub fn add_event(e: DebugEvent) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(LOG_FILE_PATH)
        .unwrap();
    let mut serialized_event = e.serialize();
    serialized_event.push('\n');
    f.write(serialized_event.as_bytes()).unwrap();
}

pub fn set_test(test_name: &String) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(LOG_FILE_PATH)
        .unwrap();
    f.write(format!("TEST_BEGIN:{}\n", test_name).as_bytes()).unwrap();
}

pub fn set_test_result(test_result: String) {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(LOG_FILE_PATH)
        .unwrap();
    f.write(format!("TEST_END:{}\n", test_result).as_bytes()).unwrap();
}

pub fn start_visuals(dslib_path: String) {
    let mut path = dslib_path;
    path.push_str(PY_CODE_PATH);
    let code = fs::read_to_string(&path).unwrap();
    let code_realpath = fs::canonicalize(&path).unwrap();
    let code_filename = code_realpath.to_str().unwrap();
    let code_module = code_filename.replace(".py", "");
    Python::with_gil(|py| {
        let debugger_module = PyModule::from_code(
            py, code.as_str(), code_filename, &code_module
        ).unwrap();
        let class = debugger_module.getattr("VDebugger").unwrap().to_object(py);
        let debugger = class.call1(py, {})
            .map_err(|e| log_python_error(e, py))
            .unwrap().to_object(py);
        debugger
            .call_method1(py, "main", (LOG_FILE_PATH,))
            .map_err(|e| log_python_error(e, py))
            .unwrap();
    });
    // remove logs
    // fs::remove_file(LOG_FILE_PATH).unwrap();
}
