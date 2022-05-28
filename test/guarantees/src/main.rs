use std::collections::HashMap;
use std::env;
use assertables::{assume, assume_eq};
use clap::{Arg, App, value_t};
use env_logger::Builder;
use log::LevelFilter;
use serde::Serialize;
use std::io::Write;
use sugars::{refcell, rc};

use std::rc::Rc;
use std::cell::RefCell;

use dslib::node::{LocalEventType, NodeActor};
use dslib::system::{System, SysEvent};
use dslib::pynode::{JsonMessage, PyNodeFactory};
use dslib::test::{TestSuite, TestResult};

use dslib::sim::{ActorId, Actor};

// UTILS -------------------------------------------------------------------------------------------

#[derive(Serialize)]
struct Info<'a> {
    info: &'a str
}

#[derive(Copy, Clone)]
struct TestConfig<'a> {
    sender_f: &'a PyNodeFactory,
    receiver_f: &'a PyNodeFactory,
    seed: u64,
    info_type: &'a str,
    reliable: bool,
    once: bool,
    ordered: bool,
}

fn init_logger(level: LevelFilter) {
    Builder::new()
        .filter(None, level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{}",
                record.args()
            )
        })
        .init();
}

fn build_system(config: &TestConfig) -> System<JsonMessage> {
    let mut sys = System::with_seed(config.seed);
    let sender = config.sender_f.build("sender", ("sender", "receiver"), config.seed);
    sys.add_node(rc!(refcell!(sender)));
    let receiver = config.receiver_f.build("receiver", ("receiver",), config.seed);
    sys.add_node(rc!(refcell!(receiver)));
    return sys;
}

fn check(delivered: &Vec<JsonMessage>, sent: &[JsonMessage],
                    config: &TestConfig) -> TestResult {
    let mut msg_count = HashMap::new();
    for msg in sent {
        msg_count.insert(msg.data.clone(), 0);
    }
    // check that delivered messages have expected type and data
    for msg in delivered.iter() {
        // assuming all messages have the same type
        assume_eq!(msg.tip, sent[0].tip, format!("Wrong message type {}", msg.tip))?;
        assume!(msg_count.contains_key(&msg.data), format!("Wrong message data: {}", msg.data))?;
        *msg_count.get_mut(&msg.data).unwrap() += 1;
    }
    // check delivered message count according to expected guarantees
    for (data, count) in msg_count {
        assume!(count > 0 || !config.reliable, format!("Message {} is not delivered", data))?;
        assume!(count < 2 || !config.once, format!("Message {} is delivered more than once", data))?;
    }
    // check message delivery order
    if config.ordered {
        let mut next_idx = 0;
        for i in 0..delivered.len() {
            let msg = &delivered[i];
            let mut matched = false;
            while !matched && next_idx < sent.len() {
                if msg.data == sent[next_idx].data {
                    matched = true;
                } else {
                    next_idx += 1;
                }
            }
            assume!(matched, format!("Order violation: {} after {}", msg.data, &delivered[i-1].data))?;
        }
    }
    Ok(true)
}

fn check_standard(sys: &mut System<JsonMessage>, sent: &[JsonMessage],
                  config: &TestConfig) -> TestResult {
    let delivered = sys.get_local_events("receiver").into_iter()
        .filter(|e| matches!(e.tip, LocalEventType::LocalMessageSend))
        .map(|e| e.msg.unwrap())
        .collect::<Vec<_>>();
    return check(&delivered, sent, config);
}

fn check_model(
    sent: &[JsonMessage],
    test_result: &mut TestResult,
    config: &TestConfig,
    actors: &HashMap<ActorId, Rc<RefCell<dyn Actor<SysEvent<JsonMessage>>>>>
) -> bool {
    let delivered = actors
        .get(&ActorId::from("receiver"))
        .unwrap()
        .borrow()
        .as_any()
        .downcast_ref::<NodeActor<JsonMessage>>()
        .unwrap()
        .get_local_events()
        .into_iter()
        .filter(|e| matches!(e.tip, LocalEventType::LocalMessageSend))
        .map(|e| e.msg.unwrap())
        .collect::<Vec<_>>();
    *test_result = check(&delivered, sent, config);
    match test_result {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn send_info_messages(sys: &mut System<JsonMessage>, info_type: &str) -> Vec<JsonMessage> {
    let infos = ["distributed", "systems", "need", "some", "guarantees"];
    let mut messages = Vec::new();
    for info in infos {
        let msg = JsonMessage::from(info_type, &Info { info });
        sys.send_local(msg.clone(), "sender");
        messages.push(msg);
    }
    return messages;
}

// TESTS -------------------------------------------------------------------------------------------

fn test_normal(config: &TestConfig, model_checking: bool) -> TestResult {
    let mut sys = build_system(config);
    let messages = send_info_messages(&mut sys, config.info_type);
    if model_checking {
        let mut test_result = Ok(true);
        sys.start_model_checking(
            &mut |actors| -> bool {
                check_model(&messages, &mut test_result, config, actors)
            },
            10,
        );
        test_result
    } else {
        sys.step_until_no_events();
        check_standard(&mut sys, &messages, config)
    }
}

fn test_delayed(config: &TestConfig, model_checking: bool) -> TestResult {
    let mut sys = build_system(config);
    sys.set_delays(1., 5.);
    let messages = send_info_messages(&mut sys, config.info_type);
    if model_checking {
        let mut test_result = Ok(true);
        sys.start_model_checking(
            &mut |actors| -> bool {
                check_model(&messages, &mut test_result, config, actors)
            },
            10,
        );
        test_result
    } else {
        sys.step_until_no_events();
        check_standard(&mut sys, &messages, config)
    }
}

fn test_duplicated(config: &TestConfig, model_checking: bool) -> TestResult {
    let mut sys = build_system(config);
    sys.set_dupl_rate(1.);
    let messages = send_info_messages(&mut sys, config.info_type);
    if model_checking {
        let mut test_result = Ok(true);
        sys.start_model_checking(
            &mut |actors| -> bool {
                check_model(&messages, &mut test_result, config, actors)
            },
            10,
        );
        test_result
    } else {
        sys.step_until_no_events();
        check_standard(&mut sys, &messages, config)
    }
}

fn test_dropped(config: &TestConfig, model_checking: bool) -> TestResult {
    let mut sys = build_system(config);
    sys.set_drop_rate(0.5);
    let messages = send_info_messages(&mut sys, config.info_type);
    if model_checking {
        let mut test_result = Ok(true);
        sys.start_model_checking(
            &mut |actors| -> bool {
                check_model(&messages, &mut test_result, config, actors)
            },
            10,
        );
        test_result
    } else {
        sys.step_until_no_events();
        check_standard(&mut sys, &messages, config)
    }
}

// MAIN --------------------------------------------------------------------------------------------

fn main() {
    let matches = App::new("Guarantees Homework Tests")
        .arg(Arg::with_name("solution_path")
            .short("i")
            .long("impl")
            .value_name("PATH")
            .help("Path to Python file with solution")
            .default_value("solution.py"))
        .arg(Arg::with_name("seed")
            .short("s")
            .long("seed")
            .value_name("SEED")
            .help("Random seed used in tests")
            .default_value("2021"))
        .arg(Arg::with_name("dslib_path")
            .short("l")
            .long("lib")
            .value_name("PATH")
            .help("Path to dslib directory")
            .default_value("../.."))
        .get_matches();
    let solution_path = matches.value_of("solution_path").unwrap();
    let seed = value_t!(matches.value_of("seed"), u64).unwrap();
    let dslib_path = matches.value_of("dslib_path").unwrap();
    init_logger(LevelFilter::Trace);

    env::set_var("PYTHONPATH", format!("{}/python", dslib_path));
    let sender_f = PyNodeFactory::new(solution_path, "Sender");
    let receiver_f = PyNodeFactory::new(solution_path, "Receiver");
    let mut config = TestConfig {
        sender_f: &sender_f,
        receiver_f: &receiver_f,
        seed,
        info_type: "INFO",
        reliable: false,
        once: false,
        ordered: false,
    };
    let mut tests = TestSuite::new();

    // At most once
    config.info_type = "INFO-1";
    config.once = true;

    // without drops should be reliable
    config.reliable = true;
    tests.add("INFO-1 NORMAL STANDARD", test_normal, config, false);
    tests.add("INFO-1 NORMAL MODEL CHECKING", test_normal, config, true);

    tests.add("INFO-1 DELAYED STANDARD", test_delayed, config, false);
    tests.add("INFO-1 DELAYED MODEL CHECKING", test_delayed, config, true);

    tests.add("INFO-1 DUPLICATED STANDARD", test_duplicated, config, false);
    tests.add("INFO-1 DUPLICATED MODEL CHECKING", test_duplicated, config, true);

    // with drops is not reliable
    config.reliable = false;
    tests.add("INFO-1 DROPPED STANDARD", test_dropped, config, false);
    tests.add("INFO-1 DROPPED MODEL CHECKING", test_dropped, config, true);

    // At least once
    config.info_type = "INFO-2";
    config.reliable = true;
    config.once = false;
    tests.add("INFO-2 NORMAL STANDARD", test_normal, config, false);
    tests.add("INFO-2 NORMAL MODEL CHECKING", test_normal, config, true);

    tests.add("INFO-2 DELAYED STANDARD", test_delayed, config, false);
    tests.add("INFO-2 DELAYED MODEL CHECKING", test_delayed, config, true);

    tests.add("INFO-2 DUPLICATED STANDARD", test_duplicated, config, false);
    tests.add("INFO-2 DUPLICATED MODEL CHECKING", test_duplicated, config, true);

    tests.add("INFO-2 DROPPED STANDARD", test_dropped, config, false);
    tests.add("INFO-2 DROPPED MODEL CHECKING", test_dropped, config, true);

    // Exactly once
    config.info_type = "INFO-3";
    config.once = true;
    tests.add("INFO-3 NORMAL STANDARD", test_normal, config, false);
    tests.add("INFO-3 NORMAL MODEL CHECKING", test_normal, config, true);

    tests.add("INFO-3 DELAYED STANDARD", test_delayed, config, false);
    tests.add("INFO-3 DELAYED MODEL CHECKING", test_delayed, config, true);

    tests.add("INFO-3 DUPLICATED STANDARD", test_duplicated, config, false);
    tests.add("INFO-3 DUPLICATED MODEL CHECKING", test_duplicated, config, true);

    tests.add("INFO-3 DROPPED STANDARD", test_dropped, config, false);
    tests.add("INFO-3 DROPPED MODEL CHECKING", test_dropped, config, true);

    // Exactly once + ordered
    config.info_type = "INFO-4";
    config.ordered = true;
    tests.add("INFO-4 NORMAL STANDARD", test_normal, config, false);
    tests.add("INFO-4 NORMAL MODEL CHECKING", test_normal, config, true);

    tests.add("INFO-4 DELAYED STANDARD", test_delayed, config, false);
    tests.add("INFO-4 DELAYED MODEL CHECKING", test_delayed, config, true);

    tests.add("INFO-4 DUPLICATED STANDARD", test_duplicated, config, false);
    tests.add("INFO-4 DUPLICATED MODEL CHECKING", test_duplicated, config, true);

    tests.add("INFO-4 DROPPED STANDARD", test_dropped, config, false);
    tests.add("INFO-4 DROPPED MODEL CHECKING", test_dropped, config, true);

    tests.run();
}