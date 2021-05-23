use self::{input::GameInput, protocol::Peer};
use super::{MatchState, MAX_PLAYERS_PER_MATCH};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

mod protocol;

pub mod backend;
pub mod input;
pub mod sync;
pub mod time_sync;

// Approximately 2 seconds of frames.
const MAX_ROLLBACK_FRAMES: usize = 120;

type Frame = i32;
const NULL_FRAME: Frame = -1;

fn is_null(frame: Frame) -> bool {
    frame < 0
}

#[derive(Copy, Clone, Debug)]
pub struct RollbackPlayerHandle(pub usize);

pub enum RollbackPlayer {
    Local,
    Spectator(Peer),
    Remote(Peer),
}

impl RollbackPlayer {
    pub fn peer(&self) -> Option<&Peer> {
        match self {
            Self::Local => None,
            Self::Remote(ref peer) => Some(peer),
            Self::Spectator(ref peer) => Some(peer),
        }
    }

    pub fn peer_mut(&mut self) -> Option<&mut Peer> {
        match self {
            Self::Local => None,
            Self::Remote(ref mut peer) => Some(peer),
            Self::Spectator(ref mut peer) => Some(peer),
        }
    }

    pub fn is_local(&self) -> bool {
        self.peer().is_none()
    }

    pub fn is_synchronized(&self) -> bool {
        if let Some(peer) = self.peer() {
            peer.state().is_synchronized()
        } else {
            true
        }
    }

    pub fn send_input<T>(&mut self, input: GameInput<T>) {
        if let Some(peer) = self.peer_mut() {
            peer.send_input(input);
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(peer) = self.peer_mut() {
            peer.disconnect();
        }
    }

    pub fn set_disconnect_timeout(&mut self, timeout: u32) {
        if let Some(peer) = self.peer_mut() {
            peer.set_disconnect_timeout(timeout);
        }
    }

    pub fn set_disconnect_notify_start(&mut self, timeout: u32) {
        if let Some(peer) = self.peer_mut() {
            peer.set_disconnect_notify_start(timeout);
        }
    }

    pub fn get_network_stats(&self) -> Option<NetworkStats> {
        self.peer().map(|peer| peer.get_network_stats())
    }
}

pub trait RollbackConfig {
    type Input: Default + Eq + Clone;
    type State;

    const MAX_PLAYERS_PER_MATCH: usize;
    const RECOMMENDATION_INTERVAL: u32;
    const DEFAULT_DISCONNECT_TIMEOUT: u32;
    const DEFAULT_DISCONNECT_NOTIFY_START: u32;
}

pub trait SessionCallbacks<T>
where
    T: RollbackConfig,
{
}

pub enum RollbackError {
    UnsupportedOperation,
    InRollback,
    NotSynchronized,
    ReachedPredictionBarrier,
    InvalidPlayer(RollbackPlayerHandle),
    PlayerDisconnected(RollbackPlayerHandle),
}

pub type RollbackResult<T> = Result<T, RollbackError>;

#[derive(Clone, Debug, Default)]
pub struct NetworkStats {
    pub ping: u32,
    pub send_queue_len: usize,
    pub recv_queue_len: usize,
    pub kbps_sent: u32,

    pub local_frames_behind: Frame,
    pub remote_frames_behind: Frame,
}

// #[derive(Clone)]
// pub struct SavedState {
//     match_state: MatchState,
//     players: [Option<SavedPlayerState>; MAX_PLAYERS_PER_MATCH],
// }

// #[derive(Clone)]
// pub struct SavedStates {
//     last_acked_state: usize,
//     states: ConstGenericRingBuffer<SavedState, MAX_ROLLBACK_FRAMES>,
// }

// impl SavedStates {
//     pub fn is_empty(&self) -> bool {
//         self.states.is_empty()
//     }
// }
