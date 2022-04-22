use std::any::Any;
use std::collections::HashSet;
use colored::*;

use crate::sim::{Actor, ActorContext};
use crate::system::{SysEvent, Message};
use crate::util::t;


pub struct Network {
    min_delay: f64,
    max_delay: f64,
    drop_rate: f64,
    dupl_rate: f64,
    corrupt_rate: f64,
    crashed_nodes: HashSet<String>,
    drop_incoming: HashSet<String>,
    drop_outgoing: HashSet<String>,
    disabled_links: HashSet<(String, String)>,
    message_count: u64,
    traffic: u64,
}

impl Network {
    pub fn new() -> Self {
        Self { 
            min_delay: 1.,
            max_delay: 1.,
            drop_rate: 0.,
            dupl_rate: 0.,
            corrupt_rate: 0.,
            crashed_nodes: HashSet::new(),
            drop_incoming: HashSet::new(),
            drop_outgoing: HashSet::new(),
            disabled_links: HashSet::new(),
            message_count: 0,
            traffic: 0,
        }
    }

    pub fn set_delay(&mut self, delay: f64) {
        self.min_delay = delay;
        self.max_delay = delay;
    }

    pub fn set_delays(&mut self, min_delay: f64, max_delay: f64) {
        self.min_delay = min_delay;
        self.max_delay = max_delay;
    }

    pub fn set_drop_rate(&mut self, drop_rate: f64) {
        self.drop_rate = drop_rate;
    }

    pub fn set_dupl_rate(&mut self, dupl_rate: f64) {
        self.dupl_rate = dupl_rate;
    }

    pub fn set_corrupt_rate(&mut self, corrupt_rate: f64) {
        self.corrupt_rate = corrupt_rate;
    }

    pub fn node_crashed(&mut self, node_id: &str) {
        self.crashed_nodes.insert(node_id.to_string());
    }

    pub fn node_recovered(&mut self, node_id: &str) {
        self.crashed_nodes.remove(node_id);
    }

    pub fn drop_incoming(&mut self, node_id: &str) {
        self.drop_incoming.insert(node_id.to_string());
    }

    pub fn pass_incoming(&mut self, node_id: &str) {
        self.drop_incoming.remove(node_id);
    }

    pub fn drop_outgoing(&mut self, node_id: &str) {
        self.drop_outgoing.insert(node_id.to_string());
    }

    pub fn pass_outgoing(&mut self, node_id: &str) {
        self.drop_outgoing.remove(node_id);
    }

    pub fn disconnect_node(&mut self, node_id: &str) {
        self.drop_incoming.insert(node_id.to_string());
        self.drop_outgoing.insert(node_id.to_string());
    }

    pub fn connect_node(&mut self, node_id: &str) {
        self.drop_incoming.remove(node_id);
        self.drop_outgoing.remove(node_id);
    }

    pub fn disable_link(&mut self, from: &str, to: &str) {
        self.disabled_links.insert((from.to_string(), to.to_string()));
    }

    pub fn enable_link(&mut self, from: &str, to: &str) {
        self.disabled_links.remove(&(from.to_string(), to.to_string()));
    }

    pub fn make_partition(&mut self, group1: &[&str], group2: &[&str]) {
        for n1 in group1 {
            for n2 in group2 {
                self.disabled_links.insert((n1.to_string(), n2.to_string()));
                self.disabled_links.insert((n2.to_string(), n1.to_string()));
            }
        }
    }

    pub fn reset_network(&mut self) {
        self.disabled_links.clear();
        self.drop_incoming.clear();
        self.drop_outgoing.clear();
    }

    pub fn get_message_count(&self) -> u64 {
        self.message_count
    }

    pub fn get_traffic(&self) -> u64 {
        self.traffic
    }
}

impl<M: Message> Actor<SysEvent<M>> for Network {
    fn on(&mut self, event: SysEvent<M>, ctx: &mut ActorContext<SysEvent<M>>, is_model_checking: bool) {
        match event {
            SysEvent::MessageSend { msg, src, dest } => {
                let msg_size = msg.size();
                if !self.crashed_nodes.contains(&src.to()) {
                    if ctx.rand() >= self.drop_rate 
                        && !self.drop_outgoing.contains(&src.to())
                        && !self.drop_incoming.contains(&dest.to())
                        && !self.disabled_links.contains(&(src.to(), dest.to())) 
                    {
                        let delay = self.min_delay + ctx.rand() * (self.max_delay - self.min_delay);
                        if ctx.rand() < self.corrupt_rate {
                            // TODO: support message corruption
                        }
                        let e = SysEvent::MessageReceive { msg, src, dest: dest.clone() };
                        if ctx.rand() >= self.dupl_rate {
                            ctx.emit(e, dest, delay);
                        } else {
                            let dups = (ctx.rand() * 2.).ceil() as u32 + 1;
                            for _i in 0..dups {
                                ctx.emit(e.clone(), dest.clone(), delay);
                            }
                        }
                    } else {
                        if !is_model_checking {
                            t!(format!("{:>9} {:>10} --x {:<10} {:?} <-- message dropped",
                                 "!!!", src.to(), dest.to(), msg).yellow());
                        }
                    }
                } else {
                    if !is_model_checking {
                        t!(format!("Discarded message from crashed node {:?}", msg).yellow());
                    }
                }
                self.message_count += 1;
                self.traffic += msg_size;
            }
            _ => (),
        }
    }

    fn is_active(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}