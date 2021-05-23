use super::{
    input::GameInput, Frame, NetworkStats, RollbackConfig, RollbackError, RollbackPlayer,
    RollbackPlayerHandle, RollbackResult,
};

mod p2p;
mod spectator;
mod sync_test;

pub use p2p::P2PBackend;
pub use sync_test::SyncTestBackend;

pub enum Backend<T>
where
    T: RollbackConfig,
{
    P2P(P2PBackend<T>),
    SyncTest(SyncTestBackend),
}

impl<T: RollbackConfig> Backend<T> {
    pub fn do_poll(&mut self, timeout: u32) -> RollbackResult<()> {
        Ok(())
    }

    pub fn add_player(&mut self, player: RollbackPlayer) -> RollbackResult<RollbackPlayerHandle> {
        match self {
            Self::P2P(p2p) => p2p.add_player(player),
            _ => Err(RollbackError::UnsupportedOperation),
        }
    }

    pub fn add_local_input(
        &mut self,
        player: RollbackPlayerHandle,
        input: GameInput<T::Input>,
    ) -> RollbackResult<()> {
        match self {
            Self::P2P(p2p) => p2p.add_local_input(player, input),
            _ => Err(RollbackError::UnsupportedOperation),
        }
    }

    pub fn sync_input(&self) -> RollbackResult<(GameInput<T::Input>, u32)> {
        match self {
            Self::P2P(p2p) => p2p.sync_input(),
            _ => Err(RollbackError::UnsupportedOperation),
        }
    }

    pub fn increment_frame(&mut self) -> RollbackResult<()> {
        match self {
            Self::P2P(p2p) => p2p.increment_frame(),
            _ => Ok(()),
        }
    }

    pub fn disconnect_player(&mut self, player: RollbackPlayerHandle) -> RollbackResult<()> {
        match self {
            Self::P2P(ref mut p2p) => p2p.disconnect_player(player),
            _ => Err(RollbackError::UnsupportedOperation),
        }
    }

    pub fn get_network_stats(&self, handle: RollbackPlayerHandle) -> RollbackResult<NetworkStats> {
        match self {
            _ => Err(RollbackError::UnsupportedOperation),
        }
    }

    pub fn set_frame_delay(
        &mut self,
        player: RollbackPlayerHandle,
        delay: Frame,
    ) -> RollbackResult<()> {
        match self {
            Self::P2P(p2p) => p2p.set_frame_delay(player, delay),
            _ => Err(RollbackError::UnsupportedOperation),
        }
    }

    pub fn set_disconnect_timeout(&mut self, timeout: u32) -> RollbackResult<()> {
        match self {
            Self::P2P(p2p) => p2p.set_disconnect_timeout(timeout),
            _ => Err(RollbackError::UnsupportedOperation),
        }
    }

    pub fn set_disconnect_notify_start(&mut self, timeout: u32) -> RollbackResult<()> {
        match self {
            Self::P2P(p2p) => p2p.set_disconnect_notify_start(timeout),
            _ => Err(RollbackError::UnsupportedOperation),
        }
    }
}
