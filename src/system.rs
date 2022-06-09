use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;
use colored::*;
use rand::prelude::*;

use crate::sim::*;
use crate::node::*;
use crate::net::*;
use crate::util::t;
use crate::pynode::{JsonMessage};

use crate::debugger;
use crate::debugger::DebugEvent;


pub trait Message: Debug + Clone {
    fn size(&self) -> u64;
    fn to_json(&self) -> JsonMessage;
}

#[derive(Debug, Clone)]
pub enum SysEvent<M: Message> {
    MessageSend {
        msg: M,
        src: ActorId,
        dest: ActorId,
    },
    MessageReceive {
        msg: M,
        src: ActorId,
        dest: ActorId,
    },
    LocalMessageReceive {
        msg: M,
    },
    TimerSet {
        name: String,
        delay: f64,
    },
    TimerFired {
        name: String,
    }
}

pub struct System<M: Message> {
    sim: Simulation<SysEvent<M>>,
    net: Rc<RefCell<Network>>,
    nodes: HashMap<String, Rc<RefCell<NodeActor<M>>>>,
    node_ids: Vec<String>,
    crashed_nodes: HashSet<String>,
}

impl<M: Message + 'static> System<M> {
    pub fn new() -> Self {
        let seed: u64 = thread_rng().gen_range(1..1_000_000);
        println!("Seed: {}", seed);
        System::with_seed(seed)
    }

    pub fn with_seed(seed: u64) -> Self {
        let mut sim = Simulation::<SysEvent<M>>::new(seed);
        let net = Rc::new(RefCell::new(Network::new()));
        sim.add_actor("net", net.clone());
        Self {
            sim,
            net,
            nodes: HashMap::new(),
            node_ids: Vec::new(),
            crashed_nodes: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, node: Rc<RefCell<dyn Node<M>>>) {
        let id = node.borrow().id().to_string();
        let actor = Rc::new(RefCell::new(NodeActor::new(node)));
        self.sim.add_actor(&id, actor.clone());
        if self.nodes.contains_key(&id) {
            if self.crashed_nodes.contains(&id) {
                // crashed node is recovered
                self.crashed_nodes.remove(&id);
                self.net.borrow_mut().node_recovered(&id);
                debugger::add_event(DebugEvent::NodeRecovered{
                    node: id.clone(),
                    ts: self.sim.time()
                });
                t!(format!("{:>9.3} {:>10} RECOVERED", self.sim.time(), &id).green().bold());
            } else {
                // node is restarted
                debugger::add_event(DebugEvent::NodeRestarted{
                    node: id.clone(),
                    ts: self.sim.time()
                });
                t!(format!("{:>9.3} {:>10} RESTARTED", self.sim.time(), &id).green().bold());
            }
        } else {
            self.node_ids.push(id.clone());
        }
        self.nodes.insert(id.clone(), actor);
    }

    pub fn get_node_ids(&self) -> Vec<String> {
        self.node_ids.clone()
    }

    pub fn set_clock_skew(&mut self, node_id: &str, clock_skew: f64) {
        let mut node = self.nodes.get(node_id).unwrap().borrow_mut();
        node.set_clock_skew(clock_skew);
    }

    pub fn crash_node(&mut self, node_id: &str) {
        debugger::add_event(DebugEvent::NodeCrashed{
            node: String::from(node_id),
            ts: self.sim.time()
        });
        t!(format!("{:>9.3} {:>10} CRASHED!", self.sim.time(), node_id).red().bold());
        self.crashed_nodes.insert(node_id.to_string());
        let mut node = self.nodes.get(node_id).unwrap().borrow_mut();
        node.crash();
        self.net.borrow_mut().node_crashed(node_id);
    }

    pub fn node_is_crashed(&self, node_id: &str) -> bool {
        self.crashed_nodes.contains(node_id)
    }

    pub fn node_count(&self) -> u32 {
        self.nodes.len() as u32
    }

    pub fn set_delay(&mut self, delay: f64) {
        self.net.borrow_mut().set_delay(delay);
    }

    pub fn set_delays(&mut self, min_delay: f64, max_delay: f64) {
        self.net.borrow_mut().set_delays(min_delay, max_delay);
    }

    pub fn set_drop_rate(&mut self, drop_rate: f64) {
        self.net.borrow_mut().set_drop_rate(drop_rate);
    }

    pub fn set_dupl_rate(&mut self, dupl_rate: f64) {
        self.net.borrow_mut().set_dupl_rate(dupl_rate);
    }

    pub fn drop_incoming(&mut self, node_id: &str) {
        t!(format!("{:>9.3} {:>10} DROPPING INCOMING", self.sim.time(), node_id).red());
        self.net.borrow_mut().drop_incoming(node_id);
    }

    pub fn pass_incoming(&mut self, node_id: &str) {
        self.net.borrow_mut().pass_incoming(node_id);
    }

    pub fn drop_outgoing(&mut self, node_id: &str) {
        self.net.borrow_mut().drop_outgoing(node_id);
    }

    pub fn pass_outgoing(&mut self, node_id: &str) {
        self.net.borrow_mut().pass_outgoing(node_id);
    }

    pub fn disconnect_node(&mut self, node_id: &str) {
        debugger::add_event(DebugEvent::NodeDisconnected{
            node: String::from(node_id),
            ts: self.sim.time()
        });
        t!(format!("{:>9.3} {:>10} DISCONNECTED", self.sim.time(), node_id).red());
        self.net.borrow_mut().disconnect_node(node_id);
    }

    pub fn connect_node(&mut self, node_id: &str) {
        debugger::add_event(DebugEvent::NodeConnected{
            node: String::from(node_id),
            ts: self.sim.time()
        });
        t!(format!("{:>9.3} {:>10} CONNECTED", self.sim.time(), node_id).green());
        self.net.borrow_mut().connect_node(node_id);
    }

    pub fn disable_link(&mut self, from: &str, to: &str) {
        debugger::add_event(DebugEvent::LinkDisabled{
            src: String::from(from),
            dst: String::from(to),
            ts: self.sim.time()
        });
        t!(format!("{:>9.3} {:>10} --> {:<10} LINK DISABLED", self.sim.time(), from, to).red());
        self.net.borrow_mut().disable_link(from, to);
    }

    pub fn enable_link(&mut self, from: &str, to: &str) {
        debugger::add_event(DebugEvent::LinkEnabled{
            src: String::from(from),
            dst: String::from(to),
            ts: self.sim.time()
        });
        t!(format!("{:>9.3} {:>10} --> {:<10} LINK ENABLED", self.sim.time(), from, to).green());
        self.net.borrow_mut().enable_link(from, to);
    }

    pub fn disable_all_links(&mut self) {
        for from in &self.node_ids {
            for to in &self.node_ids {
                if from != to {
                    self.net.borrow_mut().disable_link(from, to);
                }
            }
        }
    }

    pub fn enable_all_links(&mut self) {
        for from in &self.node_ids {
            for to in &self.node_ids {
                if from != to {
                    self.net.borrow_mut().enable_link(from, to);
                }
            }
        }
    }

    pub fn make_partition(&mut self, group1: &[&str], group2: &[&str]) {
        debugger::add_event(DebugEvent::NetworkPartition{
            group1: group1.iter().map(|&s|s.into()).collect(),
            group2: group2.iter().map(|&s|s.into()).collect(),
            ts: self.sim.time()
        });
        t!(format!("{:>9.3} NETWORK PARTITION {:?} {:?}", self.sim.time(), group1, group2).red());
        self.net.borrow_mut().make_partition(group1, group2);
    }

    pub fn reset_network(&mut self) {
        self.net.borrow_mut().reset_network();
    }

    pub fn get_network_message_count(&self) -> u64 {
        self.net.borrow().get_message_count()
    }

    pub fn get_network_traffic(&self) -> u64 {
        self.net.borrow().get_traffic()
    }

    pub fn get_sent_message_count(&self, node_id: &str) -> u64 {
        self.nodes.get(node_id).unwrap().borrow().sent_message_count()
    }

    pub fn get_received_message_count(&self, node_id: &str) -> u64 {
        self.nodes.get(node_id).unwrap().borrow().received_message_count()
    }

    pub fn send(&mut self, msg: M, src: &str, dest: &str) {
        let event = SysEvent::MessageSend {
            msg,
            src: ActorId::from(src),
            dest: ActorId::from(dest),
        };
        self.sim.add_event(event, ActorId::from(src), ActorId::from("net"), 0.0);
    }

    pub fn send_local(&mut self, msg: M, dest: &str) {
        let src = ActorId::from(&format!("local@{}", dest));
        let dest = ActorId::from(dest);
        let event = SysEvent::LocalMessageReceive { msg };
        self.sim.add_event(event, src, dest, 0.0);
    }

    pub fn time(&self) -> f64 {
        self.sim.time()
    }

    pub fn step(&mut self) -> bool {
        self.sim.step()
    }

    pub fn steps(&mut self, step_count: u32) -> bool {
        self.sim.steps(step_count)
    }

    pub fn step_until_no_events(&mut self) {
        self.sim.step_until_no_events()
    }

    pub fn step_for_duration(&mut self, duration: f64) {
        self.sim.step_for_duration(duration)
    }

    pub fn step_until_local_message(&mut self, node_id: &str) -> Result<Vec<M>,&str> {
        while self.step() {
            match self.check_mailbox(node_id) {
                Some(messages) => return Ok(messages),
                None => ()
            }
        }
        Err("No messages")
    }

    pub fn step_until_local_message_max_steps(&mut self, node_id: &str, max_steps: u32) -> Result<Vec<M>,&str> {
        let mut steps = 0;
        while self.step() && steps <= max_steps {
            match self.check_mailbox(node_id) {
                Some(messages) => return Ok(messages),
                None => ()
            }
            steps += 1;
        }
        Err("No messages")
    }

    pub fn get_local_events(&self, node_id: &str) -> Vec<LocalEvent<M>> {
        let node = self.nodes.get(node_id).unwrap().borrow();
        node.get_local_events()
    }

    pub fn check_mailbox(&mut self, node_id: &str) -> Option<Vec<M>> {
        let mut node = self.nodes.get(node_id).unwrap().borrow_mut();
        node.check_mailbox()
    }

    pub fn count_undelivered_events(&mut self) -> usize {
        self.sim.read_undelivered_events().len()
    }
}