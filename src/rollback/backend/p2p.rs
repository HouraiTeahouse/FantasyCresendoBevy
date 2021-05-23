use super::{
    super::{
        input::GameInput,
        is_null,
        protocol::ConnectionStatus,
        sync::{self, RollbackSync},
        Frame, NetworkStats, RollbackConfig, SessionCallbacks,
    },
    RollbackError, RollbackPlayer, RollbackPlayerHandle, RollbackResult,
};
use bevy::log::info;
use std::sync::{Arc, RwLock};

pub struct P2PBackend<T>
where
    T: RollbackConfig,
{
    sync: RollbackSync<T>,
    players: Vec<RollbackPlayer>,

    synchronizing: bool,
    next_recommended_sleep: u32,
    next_spectator_frame: u32,
    disconnect_timeout: u32,
    disconnect_notify_start: u32,
    local_connect_status: Arc<[RwLock<ConnectionStatus>]>,
}

impl<T: RollbackConfig> P2PBackend<T> {
    pub fn new(callbacks: Box<dyn SessionCallbacks<T>>, player_count: usize) -> Self {
        let connect_status: Vec<RwLock<ConnectionStatus>> =
            (0..player_count).map(|_| Default::default()).collect();
        let connect_status: Arc<[RwLock<ConnectionStatus>]> = connect_status.into();

        let config = sync::Config::<T> {
            callbacks,
            player_count,
        };
        let sync = RollbackSync::<T>::new(config, connect_status.clone());
        Self {
            sync,
            players: Vec::new(),
            synchronizing: true,
            next_recommended_sleep: 0,
            next_spectator_frame: 0,
            disconnect_timeout: T::DEFAULT_DISCONNECT_TIMEOUT,
            disconnect_notify_start: T::DEFAULT_DISCONNECT_NOTIFY_START,
            local_connect_status: connect_status,
        }
    }

    pub fn player_count(&self) -> usize {
        self.sync.player_count()
    }

    pub fn do_poll(&mut self, timeout: u32) -> RollbackResult<()> {
        Err(RollbackError::UnsupportedOperation)
    }

    pub fn add_player(&mut self, player: RollbackPlayer) -> RollbackResult<RollbackPlayerHandle> {
        // TODO(james7132): Ensure this does not exceed maximum number of supported players.
        let handle = RollbackPlayerHandle(self.players.len());
        self.players.push(player);
        Ok(handle)
    }

    pub fn add_local_input(
        &mut self,
        player: RollbackPlayerHandle,
        input: GameInput<T::Input>,
    ) -> RollbackResult<()> {
        if self.sync.in_rollback() {
            return Err(RollbackError::InRollback);
        }
        if self.synchronizing {
            return Err(RollbackError::NotSynchronized);
        }

        let queue = self.player_handle_to_queue(player)?;
        let frame = self.sync.add_local_input(queue, input.clone())?;
        if !is_null(frame) {
            for player in self.players.iter_mut() {
                player.send_input(input.clone())
            }
        }

        Ok(())
    }

    pub fn sync_input(&self) -> RollbackResult<(GameInput<T::Input>, u32)> {
        // Wait until we've started to return inputs.
        if self.synchronizing {
            return Err(RollbackError::NotSynchronized);
        }

        Ok(self.sync.synchronize_inputs())
    }

    pub fn increment_frame(&mut self) -> RollbackResult<()> {
        info!("End of frame ({})...", self.sync.frame_count());
        self.sync.increment_frame();
        self.do_poll(0);
        Ok(())
    }

    pub fn disconnect_player(&mut self, player: RollbackPlayerHandle) -> RollbackResult<()> {
        let queue = self.player_handle_to_queue(player)?;
        if self.local_connect_status[queue]
            .read()
            .unwrap()
            .disconnected
        {
            return Err(RollbackError::PlayerDisconnected(player));
        }

        let last_frame = self.local_connect_status[queue].read().unwrap().last_frame;
        if self.players[queue].is_local() {
            // The player is local. This should disconnect the local player from the rest
            // of the game. All other players need to be disconnected.
            // that if the endpoint is not initalized, this must be the local player.
            let current_frame = self.sync.frame_count();
            info!(
                "Disconnecting local player {} at frame {} by user request.",
                queue, last_frame
            );
            for i in 0..self.players.len() {
                if !self.players[i].is_local() {
                    self.disconnect_player_queue(i, current_frame);
                }
            }
        } else {
            info!(
                "Disconnecting queue {} at frame {} by user request.",
                queue, last_frame
            );
            self.disconnect_player_queue(queue, last_frame);
        }
        Ok(())
    }

    fn disconnect_player_queue(&mut self, queue: usize, syncto: Frame) {
        // GGPOEvent info;
        let frame_count = self.sync.frame_count();

        self.players[queue].disconnect();

        info!("Changing queue {} local connect status for last frame from {} to {} on disconnect request (current: {}).",
         queue, self.local_connect_status[queue].read().unwrap().last_frame, syncto, frame_count);

        {
            let mut status = self.local_connect_status[queue].write().unwrap();
            status.disconnected = true;
            status.last_frame = syncto;
        }

        if syncto < frame_count {
            info!(
                "Adjusting simulation to account for the fact that {} disconnected @ {}.",
                queue, syncto
            );
            self.sync.adjust_simulation(syncto);
            info!("Finished adjusting simulation.");
        }

        // info.code = GGPO_EVENTCODE_DISCONNECTED_FROM_PEER;
        // info.u.disconnected.player = QueueToPlayerHandle(queue);
        // _callbacks.on_event(&info);

        self.check_initial_sync();
    }

    pub fn get_network_stats(&self, player: RollbackPlayerHandle) -> RollbackResult<NetworkStats> {
        let queue = self.player_handle_to_queue(player)?;
        Ok(self.players[queue]
            .get_network_stats()
            .unwrap_or_else(|| Default::default()))
    }

    pub fn set_frame_delay(
        &mut self,
        player: RollbackPlayerHandle,
        delay: Frame,
    ) -> RollbackResult<()> {
        let queue = self.player_handle_to_queue(player)?;
        self.sync.set_frame_delay(queue, delay);
        Ok(())
    }

    pub fn set_disconnect_timeout(&mut self, timeout: u32) -> RollbackResult<()> {
        self.disconnect_timeout = timeout;
        for player in self.players.iter_mut() {
            if !player.is_local() {
                player.set_disconnect_timeout(timeout);
            }
        }
        Ok(())
    }

    pub fn set_disconnect_notify_start(&mut self, timeout: u32) -> RollbackResult<()> {
        self.disconnect_notify_start = timeout;
        for player in self.players.iter_mut() {
            if !player.is_local() {
                player.set_disconnect_notify_start(timeout);
            }
        }
        Ok(())
    }

    fn check_initial_sync(&mut self) {
        if self.synchronizing {
            // Check to see if everyone is now synchronized.  If so,
            // go ahead and tell the client that we're ok to accept input.
            for (i, player) in self.players.iter().enumerate() {
                if !player.is_local()
                    && !player.is_synchronized()
                    && !self.local_connect_status[i].read().unwrap().disconnected
                {
                    return;
                }
            }

            // GGPOEvent info;
            // info.code = GGPO_EVENTCODE_RUNNING;
            // _callbacks.on_event(&info);
            // _synchronizing = false;
        }
    }

    fn player_handle_to_queue(&self, player: RollbackPlayerHandle) -> RollbackResult<usize> {
        let offset = player.0;
        if offset >= self.player_count() {
            return Err(RollbackError::InvalidPlayer(player));
        }
        Ok(offset)
    }
}
