use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;
use std::any::Any;
use std::time::{Duration, SystemTime};
use decorum::R64;
use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::pynode::JsonMessage;
use crate::system::SysEvent;


#[derive(Debug, Clone)]
pub struct EventEntry<E: Debug + Clone> {
    id: u64,
    time: R64,
    src: ActorId,
    dest: ActorId,
    event: E,
}

impl<E: Debug + Clone> Eq for EventEntry<E> {}

impl<E: Debug + Clone> PartialEq for EventEntry<E> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<E: Debug + Clone> Ord for EventEntry<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
            .then_with(|| other.id.cmp(&self.id))
    }
}

impl<E: Debug + Clone> PartialOrd for EventEntry<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ActorId(String);

impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for ActorId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl ActorId {
    pub fn from(str: &str) -> Self {
        ActorId(str.to_string())
    }

    pub fn to(&self) -> String {
        self.0.clone()
    }
}

pub trait Actor<E: Debug> {
    fn on(&mut self, event: E, ctx: &mut ActorContext<E>);
    fn is_active(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn get_state(&self) -> Box<dyn Any>;
    fn set_state(&mut self, state: Box<dyn Any>);
}

pub struct CtxEvent<E> {
    event: E,
    dest: ActorId,
    delay: f64
}

pub struct ActorContext<'a, E: Debug> {
    pub id: ActorId,
    time: f64,
    rand: &'a mut Pcg64,
    next_event_id: u64,
    events: Vec<CtxEvent<E>>,
    canceled_events: Vec<u64>,
}

impl<'a, E: Debug> ActorContext<'a, E> {
    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn emit(&mut self, event: E, dest: ActorId, delay: f64) -> u64 {
        let entry = CtxEvent{ event, dest, delay };
        self.events.push(entry);
        self.next_event_id += 1;
        self.next_event_id - 1
    }

    pub fn rand(&mut self) -> f64 {
        self.rand.gen_range(0.0 .. 1.0)
    }

    pub fn cancel_event(&mut self, event_id: u64) {
        // println!("Canceled event: {}", event_id);
        self.canceled_events.push(event_id);
    }
}

pub struct Simulation<E: Debug + Clone> {
    clock: R64,
    actors: HashMap<ActorId, Rc<RefCell<dyn Actor<E>>>>,
    events: BinaryHeap<EventEntry<E>>,
    canceled_events: HashSet<u64>,
    undelivered_events: Vec<EventEntry<E>>,
    event_count: u64,
    rand: Pcg64,
    model_checking_trace: Vec<String>,
}

impl<E: 'static +  Debug + Clone> Simulation<E> {
    pub fn new(seed: u64) -> Self {        
        Self { 
            clock: R64::from_inner(0.0),
            actors: HashMap::new(),
            events: BinaryHeap::new(),
            canceled_events: HashSet::new(),
            undelivered_events: Vec::new(),
            event_count: 0,
            rand: Pcg64::seed_from_u64(seed),
            model_checking_trace: Vec::new(),
        }
    }

    pub fn time(&self) -> f64 {
        self.clock.into_inner()
    }

    pub fn add_actor(&mut self, id: &str, actor: Rc<RefCell<dyn Actor<E>>>) {
        self.actors.insert(ActorId(id.to_string()), actor);
    }

    pub fn add_event(
        &mut self,
        event: E,
        src: ActorId,
        dest: ActorId,
        delay: f64,
        mc_events: Option<&mut Vec<EventEntry<E>>>,
    ) -> u64 {
        let entry = EventEntry {
            id: self.event_count,
            time: self.clock + delay,
            src, dest, event
        };
        let id = entry.id;
        match mc_events {
            None => self.events.push(entry),
            Some(events_vec) => events_vec.push(entry),
        }
        self.event_count += 1;
        id
    }

    pub fn cancel_event(&mut self, event_id: u64) {
        self.canceled_events.insert(event_id);
    }

    pub fn step(&mut self, mut mc_events: Option<&mut Vec<EventEntry<E>>>) -> bool {
        if let Some(e) = if let Some(events_vec) = mc_events.as_mut().map(|x| &mut **x) {
            events_vec.pop()
        } else {
            self.events.pop()
        } {
            if !self.canceled_events.remove(&e.id) {
                // println!("{} {}->{} {:?}", e.time, e.src, e.dest, e.event);
                self.clock = e.time;
                let actor = self.actors.get(&e.dest);
                let mut ctx = ActorContext{
                    id: e.dest.clone(), 
                    time: self.clock.into_inner(), 
                    rand: &mut self.rand, 
                    next_event_id: self.event_count,
                    events: Vec::new(),
                    canceled_events: Vec::new(),
                };
                match actor {
                    Some(actor) => {
                        if actor.borrow().is_active() {
                            actor.borrow_mut().on(e.event, &mut ctx);
                            let canceled = ctx.canceled_events.clone();
                            for ctx_e in ctx.events {
                                self.add_event(
                                    ctx_e.event,
                                    e.dest.clone(),
                                    ctx_e.dest,
                                    ctx_e.delay,
                                    mc_events.as_mut().map(|x| &mut **x),
                                );
                            };
                            for event_id in canceled {
                                self.cancel_event(event_id);
                            };
                        } else {
                            //println!("Discarded event for inactive actor {}", e.dest);
                        }
                    }
                    _ => {
                        self.undelivered_events.push(e);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    pub fn steps(&mut self, step_count: u32) -> bool {
        for _i in 0..step_count {
            if !self.step(None) {
                return false
            }
        }
        true
    }

    pub fn step_until_no_events(&mut self) {
        while self.step(None) {
        }
    }

    pub fn step_for_duration(&mut self, duration: f64) {
        let end_time = self.time() + duration;
        while self.step(None) && self.time() < end_time {
        }
    }

    pub fn model_checking_step(
        &mut self,
        check_fn: &mut dyn for<'r> FnMut(&'r HashMap<ActorId, Rc<RefCell<dyn Actor<E>>>>) -> bool,
        sys_time: &SystemTime,
        limit_seconds: u64,
        events: &mut Vec<EventEntry<E>>,
    ) -> bool {
        let mc_events_count = events.len();
        if mc_events_count == 0 {
            return check_fn(&self.actors);
        }
        for i in 0..mc_events_count {
            if sys_time.elapsed().unwrap() >= Duration::from_secs(limit_seconds) {
                return true;
            }
            let mut actors_states: HashMap<ActorId, Box<dyn Any>> = HashMap::new();
            for (actor_id, actor) in &self.actors {
                actors_states.insert(actor_id.clone(), actor.borrow().get_state());
            }
            let event_count = self.event_count;
            let canceled_events = self.canceled_events.clone();
            let rand = self.rand.clone();
            let event = events.remove(i);
            events.push(event.clone());
            self.step(Some(events));
            let next_step_res = self.model_checking_step(check_fn, sys_time, limit_seconds, events);
            if !next_step_res {
                let event_e = event.event.clone();
                let event_any = &event_e as &dyn Any;
                let (event_type, event_text1, event_text2) = if let Some(sys_event) = event_any.downcast_ref::<SysEvent<JsonMessage>>() {
                    match sys_event {
                        SysEvent::MessageSend { msg, src: _, dest: _ } => {
                            ("message_send", &((*msg).tip[..]), &((*msg).data[..]))
                        }
                        SysEvent::MessageReceive { msg, src: _, dest: _ } => {
                            ("message_receive", &((*msg).tip[..]), &((*msg).data[..]))
                        }
                        SysEvent::LocalMessageReceive { msg } => {
                            ("local_message_receive", &((*msg).tip[..]), &((*msg).data[..]))
                        }
                        SysEvent::TimerSet { name, delay: _ } => {
                            ("timer_set", &(name[..]), "")
                        }
                        SysEvent::TimerFired { name } => {
                            ("timer_fired", &(name[..]), "")
                        }
                    }
                } else {
                    ("", "", "")
                };
                self.model_checking_trace.push(format!(
                    "{:>9.3} {:>15} --> {:<15} {:^25} {:<10} {:?}",
                    event.time,
                    event.src.to_string(),
                    event.dest.to_string(),
                    event_type,
                    event_text1,
                    event_text2,
                ));
            }
            while events.len() >= mc_events_count {
                events.pop();
            }
            events.insert(i, event);
            self.canceled_events = canceled_events;
            self.event_count = event_count;
            self.rand = rand;
            for (actor_id, actor_state) in actors_states {
                self.actors[&actor_id].borrow_mut().set_state(actor_state);
            }
            if !next_step_res {
                return false;
            }
        }
        return true;
    }

    pub fn run_model_checking(
        &mut self,
        check_fn: &mut dyn for<'r> FnMut(&'r HashMap<ActorId, Rc<RefCell<dyn Actor<E>>>>) -> bool,
        sys_time: &SystemTime,
        limit_seconds: u64,
    ) -> bool {
        let mut events = self.events.clone().into_vec();
        return self.model_checking_step(check_fn, sys_time, limit_seconds, &mut events);
    }

    pub fn read_undelivered_events(&mut self) -> Vec<EventEntry<E>> {
        self.undelivered_events.drain(..).collect()
    }
    
    pub fn read_model_checking_trace(&mut self) -> Vec<String> {
        self.model_checking_trace.drain(..).rev().collect()
    }
}
