use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use crate::sim::{Actor, ActorId, ActorContext};
use crate::system::SysEvent;


pub trait Node<M: Debug + Clone> {
    fn id(&self) -> &String;
    fn on_message(&mut self, msg: M, from: String, ctx: &mut Context<M>);
    fn on_local_message(&mut self, msg: M, ctx: &mut Context<M>);
    fn on_timer(&mut self, timer: String, ctx: &mut Context<M>);
}

pub struct Context<'a, 'b, 'c, M: Debug + Clone> {
    ctx: &'a mut ActorContext<'b, SysEvent<M>>,
    timers: &'c mut HashMap<(ActorId, String), u64>,
    local_messages: &'c mut Vec<M>,
}

impl<'a, 'b, 'c, M: Debug + Clone> Context<'a, 'b, 'c, M> {
    pub fn new(
        ctx: &'a mut ActorContext<'b, SysEvent<M>>,
        timers: &'c mut HashMap<(ActorId, String), u64>,
        local_messages: &'c mut Vec<M>,
    ) -> Self {
        Self {
            ctx,
            timers,
            local_messages,
        }
    }

    pub fn time(&self) -> f64 {
        self.ctx.time()
    }

    pub fn send(&mut self, msg: M, dest: &str) {
        let dest = ActorId::from(dest);
        println!("{:>9.3} {:>10} --> {:<10} {:?}", self.ctx.time(), self.ctx.id.to(), dest.to(), msg);
        if self.ctx.id == dest {
            let event = SysEvent::MessageReceive { msg, src: self.ctx.id.clone(), dest: dest.clone() };
            self.ctx.emit(event, dest, 0.0);
        } else {
            let event = SysEvent::MessageSend { msg, src: self.ctx.id.clone(), dest };
            self.ctx.emit(event, ActorId::from("net"), 0.0);
        }
    }

    pub fn send_local(&mut self, msg: M) {
        println!("{:>9.3} {:>10} >>> {:<10} {:?}", self.ctx.time(), self.ctx.id.to(), "local", msg);
        self.local_messages.push(msg);
    }

    pub fn set_timer(&mut self, name: &str, delay: f64) {
        let event = SysEvent::TimerFired { name: name.to_string() };
        let event_id = self.ctx.emit(event, self.ctx.id.clone(), delay);
        self.timers.insert((self.ctx.id.clone(), name.to_string()), event_id);
    }

    pub fn cancel_timer(&mut self, name: &str) {
        match self.timers.remove(&(self.ctx.id.clone(), name.to_string())) {
            Some(event_id) => self.ctx.cancel_event(event_id),
            _ => ()
        }
    }

    pub fn rand(&mut self) -> f64 {
        self.ctx.rand()
    }
}

enum NodeSatus {
    HEALTHY,
    CRASHED,
}

pub struct NodeActor<M> {
    node: Rc<RefCell<dyn Node<M>>>,
    timers: HashMap<(ActorId, String), u64>,
    local_messages: Vec<M>,
    status: NodeSatus,
}

impl<M> NodeActor<M> {
    pub fn new(node: Rc<RefCell<dyn Node<M>>>) -> Self {
        Self { 
            node,
            timers: HashMap::new(),
            local_messages: Vec::new(),
            status: NodeSatus::HEALTHY,
        }
    }

    pub fn count_local_messages(&self) -> usize {
        self.local_messages.len()
    }

    pub fn read_local_messages(&mut self) -> Vec<M> {
        let mut messages = Vec::new();
        for msg in self.local_messages.drain(..) {
            messages.push(msg);
        }
        messages
    }

    pub fn crash(&mut self) {
        self.status = NodeSatus::CRASHED;
    }
}

impl<M: Debug + Clone> Actor<SysEvent<M>> for NodeActor<M> {
    fn on(&mut self, event: SysEvent<M>, ctx: &mut ActorContext<SysEvent<M>>) {
        match self.status {
            NodeSatus::HEALTHY => {
                match event {
                    SysEvent::MessageReceive { msg, src, dest } => {
                        println!("{:>9.3} {:>10} <-- {:<10} {:?}", ctx.time(), dest.to(), src.to(), msg);
                        let mut node_ctx = Context::new(ctx, &mut self.timers, &mut self.local_messages);
                        self.node.borrow_mut().on_message(msg, src.to(), &mut node_ctx);
                    }
                    SysEvent::LocalMessageReceive { msg } => {
                        println!("{:>9.3} {:>10} <<< {:<10} {:?}", ctx.time(), ctx.id.to(), "local", msg);
                        let mut node_ctx = Context::new(ctx, &mut self.timers, &mut self.local_messages);
                        self.node.borrow_mut().on_local_message(msg, &mut node_ctx);
                    }
                    SysEvent::TimerFired { name } => {
                        println!("{:>9.3} {:>10} -!- {:<10}", ctx.time(), ctx.id.to(), name);
                        self.timers.remove(&(ctx.id.clone(), name.clone()));
                        let mut node_ctx = Context::new(ctx, &mut self.timers, &mut self.local_messages);
                        self.node.borrow_mut().on_timer(name, &mut node_ctx);
                    }
                    _ => ()
                }        
            }
            NodeSatus::CRASHED => ()
        }
    }

    fn is_active(&self) -> bool {
        matches!(self.status, NodeSatus::HEALTHY)
    }
}