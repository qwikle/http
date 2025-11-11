use std::sync::atomic::{AtomicU8, Ordering};

use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerState {
    Booting = 0,
    Ready = 1,
    Run = 2,
    Closing = 3,
    Shutdown = 4
}

pub struct LifecycleManager {
    state: AtomicU8,
    state_sender: broadcast::Sender<ServerState>
}

impl LifecycleManager {
    pub fn new() -> Self {
        let (state_sender, _)= broadcast::channel(16);
        let manager = Self {
            state: AtomicU8::new(ServerState::Booting as u8),
            state_sender
        };
        let _ = manager.state_sender.send(ServerState::Booting);
        manager
    }
    
    pub fn transition_to(&self, state: ServerState) -> Result<(), &'static str> {
        let current = self.curent_state();
        match (current, state) {
            (ServerState::Booting, ServerState::Ready) => {},
            (ServerState::Ready, ServerState::Run) => {},
            (ServerState::Run, ServerState::Closing) => {},
            (ServerState::Closing, ServerState::Shutdown) => {},
            _ => return Err("Transition invalid"),
        }
        self.state.store(state as u8, Ordering::SeqCst);
        let _ = self.state_sender.send(state);
        Ok(())
    }
    
    pub fn curent_state(&self) -> ServerState {
        match self.state.load(Ordering::SeqCst) {
            0 => ServerState::Booting,
            1 => ServerState::Ready,
            2 => ServerState::Run,
            3 => ServerState::Closing,
            4 => ServerState::Shutdown,
            _ => unreachable!()
        }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ServerState> {
        self.state_sender.subscribe()
    }
}