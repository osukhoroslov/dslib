use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;
use decorum::R64;
use rand::prelude::*;
use rand_pcg::Pcg64;


#[derive(Debug)]
pub struct EventEntry<E: Debug> {
    id: u64,
    time: R64,
    src: ActorId,
    dest: ActorId,
    event: E,
}

impl<E: Debug> Eq for EventEntry<E> {}

impl<E: Debug> PartialEq for EventEntry<E> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<E: Debug> Ord for EventEntry<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
            .then_with(|| other.id.cmp(&self.id))
    }
}

impl<E: Debug> PartialOrd for EventEntry<E> {
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

pub struct Simulation<E: Debug> {
    clock: R64,
    actors: HashMap<ActorId, Rc<RefCell<dyn Actor<E>>>>,
    events: BinaryHeap<EventEntry<E>>,
    canceled_events: HashSet<u64>,
    undelivered_events: Vec<EventEntry<E>>,
    event_count: u64,
    rand: Pcg64,
}

impl<E: Debug> Simulation<E> {
    pub fn new(seed: u64) -> Self {        
        Self { 
            clock: R64::from_inner(0.0),
            actors: HashMap::new(),
            events: BinaryHeap::new(),
            canceled_events: HashSet::new(),
            undelivered_events: Vec::new(),
            event_count: 0,
            rand: Pcg64::seed_from_u64(seed),
        }
    }

    pub fn time(&self) -> f64 {
        self.clock.into_inner()
    }

    pub fn add_actor(&mut self, id: &str, actor: Rc<RefCell<dyn Actor<E>>>) {
        self.actors.insert(ActorId(id.to_string()), actor);
    }

    pub fn add_event(&mut self, event: E, src: ActorId, dest: ActorId, delay: f64) -> u64 {
        let entry = EventEntry {
            id: self.event_count,
            time: self.clock + delay,
            src, dest, event
        };
        let id = entry.id;
        self.events.push(entry);
        self.event_count += 1;
        id
    }

    pub fn cancel_event(&mut self, event_id: u64) {
        self.canceled_events.insert(event_id);
    }

    pub fn step(&mut self) -> bool {
        if let Some(e) = self.events.pop() {
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
                                self.add_event(ctx_e.event, e.dest.clone(), ctx_e.dest, ctx_e.delay);
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

    pub fn steps(&mut self, step_count: u32) {
        for _i in 0..step_count {
            self.step();
        }
    }

    pub fn step_until_no_events(&mut self) {
        while self.step() {
        }
    }

    pub fn read_undelivered_events(&mut self) -> Vec<EventEntry<E>> {
        self.undelivered_events.drain(..).collect()
    }
}
