//#[cfg(feature = "std")]
//pub mod llmp;

use alloc::string::String;
use core::marker::PhantomData;

use serde::{Deserialize, Serialize};

//#[cfg(feature = "std")]
//pub mod llmp_translated; // TODO: Abstract away.
//#[cfg(feature = "std")]
//pub mod shmem_translated;

//#[cfg(feature = "std")]
//pub use crate::events::llmp::LLMPEventManager;

#[cfg(feature = "std")]
use std::io::Write;

use crate::corpus::Corpus;
use crate::engines::State;
use crate::executors::Executor;
use crate::inputs::Input;
use crate::serde_anymap::{Ptr, PtrMut};
use crate::utils::Rand;
use crate::AflError;

/// Indicate if an event worked or not
pub enum BrokerEventResult {
    /// The broker haneled this. No need to pass it on.
    Handled,
    /// Pass this message along to the clients.
    Forward,
}

pub trait ShowStats {

}

/*

/// A custom event, in case a user wants to extend the features (at compile time)
pub trait CustomEvent<C, E, I, R>
where
    S: State<I, R>,
    C: Corpus<I, R>,
    E: Executor<I>,
    I: Input,
    R: Rand,
{
    /// Returns the name of this event
    fn name(&self) -> &str;
    /// This method will be called in the broker
    fn handle_in_broker(&self, broker: &dyn EventManager<C, E, I, R, Self>, state: &mut State<I, R>, corpus: &mut C) -> Result<BrokerEventResult, AflError>;
    /// This method will be called in the clients after handle_in_broker (unless BrokerEventResult::Handled) was returned in handle_in_broker
    fn handle_in_client(&self, client: &dyn EventManager<C, E, I, R, Self>, state: &mut State<I, R>, corpus: &mut C) -> Result<(), AflError>;
}

struct UnusedCustomEvent {}
impl<C, E, I, R> CustomEvent<C, E, I, R> for UnusedCustomEvent<C, E, I, R>
where
    S: State<I, R>,
    C: Corpus<I, R>,
    E: Executor<I>,
    I: Input,
    R: Rand,
{
    fn name(&self) -> &str {"No custom events"}
    fn handle_in_broker(&self, broker: &dyn EventManager<C, E, I, R, Self>, state: &mut State<I, R>, corpus: &mut C) {Ok(BrokerEventResult::Handled)}
    fn handle_in_client(&self, client: &dyn EventManager<C, E, I, R, Self>, state: &mut State<I, R>, corpus: &mut C) {Ok(())}
}
*/

/// Events sent around in the library
#[derive(Serialize, Deserialize)]
pub enum Event<'a, C, E, I, R>
where
    C: Corpus<I, R>,
    E: Executor<I>,
    I: Input,
    R: Rand,
    // CE: CustomEvent<C, E, I, R>,
{
    LoadInitial {
        sender_id: u64,
        phantom: PhantomData<(C, E, I, R)>,
    },
    NewTestcase {
        sender_id: u64,
        input: Ptr<'a, I>,
        observers: PtrMut<'a, crate::observers::observer_serde::NamedSerdeAnyMap>,
    },
    UpdateStats {
        sender_id: u64,
        executions: usize,
        execs_over_sec: u64,
        phantom: PhantomData<(C, E, I, R)>,
    },
    Crash {
        sender_id: u64,
        input: I,
        phantom: PhantomData<(C, E, I, R)>,
    },
    Timeout {
        sender_id: u64,
        input: I,
        phantom: PhantomData<(C, E, I, R)>,
    },
    Log {
        sender_id: u64,
        severity_level: u8,
        message: String,
        phantom: PhantomData<(C, E, I, R)>,
    },
    None {
        phantom: PhantomData<(C, E, I, R)>,
    },
    //Custom {sender_id: u64, custom_event: CE},
}

impl<'a, C, E, I, R> Event<'a, C, E, I, R>
where
    C: Corpus<I, R>,
    E: Executor<I>,
    I: Input,
    R: Rand,
    //CE: CustomEvent<C, E, I, R>,
{
    pub fn name(&self) -> &str {
        match self {
            Event::LoadInitial {
                sender_id: _,
                phantom: _,
            } => "Initial",
            Event::NewTestcase {
                sender_id: _,
                input: _,
                observers: _,
            } => "New Testcase",
            Event::UpdateStats {
                sender_id: _,
                executions: _,
                execs_over_sec: _,
                phantom: _,
            } => "Stats",
            Event::Crash {
                sender_id: _,
                input: _,
                phantom: _,
            } => "Crash",
            Event::Timeout {
                sender_id: _,
                input: _,
                phantom: _,
            } => "Timeout",
            Event::Log {
                sender_id: _,
                severity_level: _,
                message: _,
                phantom: _,
            } => "Log",
            Event::None { phantom: _ } => "None",
            //Event::Custom {sender_id, custom_event} => custom_event.name(),
        }
    }

    pub fn log(severity_level: u8, message: String) -> Self {
        Event::Log {
            sender_id: 0,
            severity_level: severity_level,
            message: message,
            phantom: PhantomData,
        }
    }

    pub fn update_stats(executions: usize, execs_over_sec: u64) -> Self {
        Event::UpdateStats {
            sender_id: 0,
            executions: executions,
            execs_over_sec: execs_over_sec,
            phantom: PhantomData,
        }
    }

    // TODO serialize and deserialize, defaults to serde
}

pub trait EventManager<C, E, I, R>
where
    C: Corpus<I, R>,
    E: Executor<I>,
    I: Input,
    R: Rand,
{
    /// Check if this EventaManager support a given Event type
    /// To compare events, use Event::name().as_ptr()
    fn enabled(&self) -> bool;

    /// Fire an Event
    fn fire<'a>(
        &mut self,
        event: Event<'a, C, E, I, R>,
        state: &mut State<I, R>,
        corpus: &mut C,
    ) -> Result<(), AflError>;

    /// Lookup for incoming events and process them.
    /// Return the number of processes events or an error
    fn process(&mut self, state: &mut State<I, R>, corpus: &mut C) -> Result<usize, AflError>;

    fn on_recv(&self, _state: &mut State<I, R>, _corpus: &mut C) -> Result<(), AflError> {
        // TODO: Better way to move out of testcase, or get ref
        //Ok(corpus.add(self.testcase.take().unwrap()))
        Ok(())
    }

    // TODO the broker has a state? do we need to pass state and corpus?
    fn handle_in_broker(
        &self,
        event: &Event<C, E, I, R>,
        /*broker: &dyn EventManager<C, E, I, R>,*/ _state: &mut State<I, R>,
        _corpus: &mut C,
    ) -> Result<BrokerEventResult, AflError> {
        match event {
            Event::LoadInitial {
                sender_id: _,
                phantom: _,
            } => Ok(BrokerEventResult::Handled),
            Event::NewTestcase {
                sender_id: _,
                input: _,
                observers: _,
            } => Ok(BrokerEventResult::Forward),
            Event::UpdateStats {
                sender_id: _,
                executions: _,
                execs_over_sec: _,
                phantom: _,
            } => {
                // TODO
                Ok(BrokerEventResult::Handled)
            }
            Event::Crash {
                sender_id: _,
                input: _,
                phantom: _,
            } => Ok(BrokerEventResult::Handled),
            Event::Timeout {
                sender_id: _,
                input: _,
                phantom: _,
            } => {
                // TODO
                Ok(BrokerEventResult::Handled)
            }
            Event::Log {
                sender_id,
                severity_level,
                message,
                phantom: _,
            } => {
                //TODO: broker.log()
                #[cfg(feature = "std")]
                println!("{}[{}]: {}", sender_id, severity_level, message);
                Ok(BrokerEventResult::Handled)
            },
            Event::None {
                phantom: _,
            } => Ok(BrokerEventResult::Handled)
            //Event::Custom {sender_id, custom_event} => custom_event.handle_in_broker(state, corpus),
            //_ => Ok(BrokerEventResult::Forward),
        }
    }

    fn handle_in_client(
        &self,
        event: Event<C, E, I, R>,
        /*client: &dyn EventManager<C, E, I, R>,*/ _state: &mut State<I, R>,
        corpus: &mut C,
    ) -> Result<(), AflError> {
        match event {
            Event::NewTestcase {
                sender_id: _,
                input: _,
                observers: _,
            } => {
                // here u should match sender_id, if equal to the current one do not re-execute
                // we need to pass engine to process() too, TODO
                #[cfg(feature = "std")]
                println!("PLACEHOLDER: received NewTestcase");
                Ok(())
            }
            _ => Err(AflError::Unknown(
                "Received illegal message that message should not have arrived.".into(),
            )),
        }
    }
}

/*TODO
    fn on_recv(&self, state: &mut State<I, R>, _corpus: &mut C) -> Result<(), AflError> {
        println!(
            "#{}\t exec/s: {}",
            state.executions(),
            //TODO: Count corpus.entries().len(),
            state.executions_over_seconds()
        );
        Ok(())
    }
*/

#[cfg(feature = "std")]
pub struct LoggerEventManager<C, E, I, R, W>
where
    W: Write,
    //CE: CustomEvent<C, E, I, R>,
{
    writer: W,
    count: usize,
    phantom: PhantomData<(C, E, I, R)>,
}

#[cfg(feature = "std")]
impl<C, E, I, R, W> EventManager<C, E, I, R> for LoggerEventManager<C, E, I, R, W>
where
    C: Corpus<I, R>,
    E: Executor<I>,
    I: Input,
    R: Rand,
    W: Write,
    //CE: CustomEvent<C, E, I, R>,
{
    fn enabled(&self) -> bool {
        true
    }

    fn fire<'a>(
        &mut self,
        event: Event<'a, C, E, I, R>,
        state: &mut State<I, R>,
        corpus: &mut C,
    ) -> Result<(), AflError> {
        match self.handle_in_broker(&event, state, corpus)? {
            BrokerEventResult::Forward => (), //self.handle_in_client(event, state, corpus)?,
            // Ignore broker-only events
            BrokerEventResult::Handled => (),
        }
        Ok(())
    }

    fn process(&mut self, _state: &mut State<I, R>, _corpus: &mut C) -> Result<usize, AflError> {
        let c = self.count;
        self.count = 0;
        Ok(c)
    }
}

#[cfg(feature = "std")]
impl<C, E, I, R, W> LoggerEventManager<C, E, I, R, W>
where
    C: Corpus<I, R>,
    I: Input,
    E: Executor<I>,
    R: Rand,
    W: Write,
    //TODO CE: CustomEvent,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer: writer,
            count: 0,
            phantom: PhantomData,
        }
    }
}
