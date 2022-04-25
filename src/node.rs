use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use colored::*;

use crate::sim::{Actor, ActorId, ActorContext};
use crate::system::{Message, SysEvent};
use crate::util::t;


pub trait Node<M: Message> {
    fn id(&self) -> &String;
    fn on_message(&mut self, msg: M, from: String, ctx: &mut Context<M>);
    fn on_local_message(&mut self, msg: M, ctx: &mut Context<M>);
    fn on_timer(&mut self, timer: String, ctx: &mut Context<M>);
    fn get_state(&mut self) -> String;
    fn set_state(&mut self, json_state: String);
}

pub struct Context<'a, 'b, 'c, M: Message> {
    ctx: &'a mut ActorContext<'b, SysEvent<M>>,
    timers: &'c mut HashMap<(ActorId, String), u64>,
    local_events: &'c mut Vec<LocalEvent<M>>,
    local_mailbox: &'c mut Vec<M>,
    sent_message_count: &'c mut u64,
    clock_skew: f64,
}

impl<'a, 'b, 'c, M: Message> Context<'a, 'b, 'c, M> {
    pub fn new(
        ctx: &'a mut ActorContext<'b, SysEvent<M>>,
        timers: &'c mut HashMap<(ActorId, String), u64>,
        local_events: &'c mut Vec<LocalEvent<M>>,
        local_mailbox: &'c mut Vec<M>,
        sent_message_count: &'c mut u64,
        clock_skew: f64,
    ) -> Self {
        Self {
            ctx,
            timers,
            local_events,
            local_mailbox,
            sent_message_count,
            clock_skew,
        }
    }

    pub fn time(&mut self) -> f64 {
        self.ctx.time() + self.clock_skew
    }

    pub fn send(&mut self, msg: M, dest: &str) {
        let dest = ActorId::from(dest);
        t!("{:>9.3} {:>10} --> {:<10} {:?}", self.ctx.time(), self.ctx.id.to(), dest.to(), msg);
        if self.ctx.id == dest {
            let event = SysEvent::MessageReceive { msg, src: self.ctx.id.clone(), dest: dest.clone() };
            self.ctx.emit(event, dest, 0.0);
        } else {
            let event = SysEvent::MessageSend { msg, src: self.ctx.id.clone(), dest };
            self.ctx.emit(event, ActorId::from("net"), 0.0);
        }
        *self.sent_message_count += 1;
    }

    pub fn send_local(&mut self, msg: M) {
        t!(format!("{:>9.3} {:>10} >>> {:<10} {:?}", self.ctx.time(), self.ctx.id.to(), "local", msg).cyan());
        let event = LocalEvent {
            time: self.time(),
            msg: Some(msg.clone()),
            tip: LocalEventType::LocalMessageSend
        };
        self.local_events.push(event);
        self.local_mailbox.push(msg);
    }

    pub fn set_timer(&mut self, name: &str, delay: f64) {
        let event = SysEvent::TimerFired { name: name.to_string() };
        let event_id = self.ctx.emit(event, self.ctx.id.clone(), delay);
        self.timers.insert((self.ctx.id.clone(), name.to_string()), event_id);
    }

    pub fn cancel_timer(&mut self, name: &str) {
        if let Some(event_id) = self.timers.remove(&(self.ctx.id.clone(), name.to_string())) {
            self.ctx.cancel_event(event_id);
        }
    }

    pub fn rand(&mut self) -> f64 {
        self.ctx.rand()
    }
}

#[derive(Debug, Clone)]
pub enum LocalEventType {
    LocalMessageSend,
    LocalMessageReceive,
}

#[derive(Debug, Clone)]
pub struct LocalEvent<M: Message> {
    pub time: f64,
    pub msg: Option<M>,
    pub tip: LocalEventType
}

enum NodeStatus {
    Healthy,
    Crashed,
}

pub struct NodeActor<M: Message> {
    node: Rc<RefCell<dyn Node<M>>>,
    timers: HashMap<(ActorId, String), u64>,
    local_events: Vec<LocalEvent<M>>,
    local_mailbox: Vec<M>,
    status: NodeStatus,
    sent_message_count: u64,
    received_message_count: u64,
    clock_skew: f64,
}

impl<M: Message> NodeActor<M> {
    pub fn new(node: Rc<RefCell<dyn Node<M>>>) -> Self {
        Self {
            node,
            timers: HashMap::new(),
            local_events: Vec::new(),
            local_mailbox: Vec::new(),
            status: NodeStatus::Healthy,
            sent_message_count: 0,
            received_message_count: 0,
            clock_skew: 0.0,
        }
    }

    pub fn get_local_events(&self) -> Vec<LocalEvent<M>> {
        self.local_events.clone()
    }

    pub fn check_mailbox(&mut self) -> Option<Vec<M>> {
        if self.local_mailbox.len() > 0 {
            Some(self.local_mailbox.drain(..).collect())
        } else {
            None
        }
    }

    pub fn sent_message_count(&self) -> u64 {
        self.sent_message_count
    }

    pub fn received_message_count(&self) -> u64 {
        self.received_message_count
    }

    pub fn crash(&mut self) {
        self.status = NodeStatus::Crashed;
    }

    pub fn set_clock_skew(&mut self, clock_skew: f64) {
        self.clock_skew = clock_skew
    }
}

impl<M: 'static +  Message> Actor<SysEvent<M>> for NodeActor<M> {
    fn on(&mut self, event: SysEvent<M>, ctx: &mut ActorContext<SysEvent<M>>) {
        match self.status {
            NodeStatus::Healthy => {
                match event {
                    SysEvent::MessageReceive { msg, src, dest } => {
                        t!("{:>9.3} {:>10} <-- {:<10} {:?}", ctx.time(), dest.to(), src.to(), msg);
                        let mut node_ctx = Context::new(
                            ctx, &mut self.timers, &mut self.local_events, &mut self.local_mailbox,
                            &mut self.sent_message_count, self.clock_skew);
                        self.node.borrow_mut().on_message(msg, src.to(), &mut node_ctx);
                        self.received_message_count += 1;
                    }
                    SysEvent::LocalMessageReceive { msg } => {
                        t!(format!("{:>9.3} {:>10} <<< {:<10} {:?}", ctx.time(), ctx.id.to(), "local", msg).cyan());
                        self.local_events.push(LocalEvent {
                            time: ctx.time(),
                            msg: Some(msg.clone()),
                            tip: LocalEventType::LocalMessageReceive
                        });
                        let mut node_ctx = Context::new(
                            ctx, &mut self.timers, &mut self.local_events, &mut self.local_mailbox,
                            &mut self.sent_message_count, self.clock_skew);
                        self.node.borrow_mut().on_local_message(msg, &mut node_ctx);
                    }
                    SysEvent::TimerFired { name } => {
                        t!(format!("{:>9.3} {:>10} !-- {:<10}", ctx.time(), ctx.id.to(), name).magenta());
                        self.timers.remove(&(ctx.id.clone(), name.clone()));
                        let mut node_ctx = Context::new(
                            ctx, &mut self.timers, &mut self.local_events, &mut self.local_mailbox,
                            &mut self.sent_message_count, self.clock_skew);
                        self.node.borrow_mut().on_timer(name, &mut node_ctx);
                    }
                    _ => ()
                }
            }
            NodeStatus::Crashed => ()
        }
    }

    fn is_active(&self) -> bool {
        matches!(self.status, NodeStatus::Healthy)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
