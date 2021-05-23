use super::{input::GameInput, Frame, NetworkStats};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const MSG_MAX_PLAYERS: usize = 8;

#[derive(Clone, Debug)]
pub enum PeerState {
    Syncing {
        roundtrips_remaining: u32,
        random: u32,
    },
    Synchronzied,
    Running {
        last_quality_report_time: u32,
        last_network_stats_interval: u32,
        last_input_packet_recv_time: u32,
    },
    Disconnected,
}

impl PeerState {
    pub fn is_synchronized(&self) -> bool {
        if let Self::Running { .. } = self {
            true
        } else {
            false
        }
    }
}

impl Default for PeerState {
    fn default() -> Self {
        Self::Syncing {
            roundtrips_remaining: 0,
            random: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MessageQueue(VecDeque<Message>);

impl MessageQueue {
    pub fn send(&mut self, msg: Message) {
        self.0.push_front(msg);
    }

    pub fn recieve(&mut self) -> Option<Message> {
        self.0.pop_back()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Default, Debug)]
pub struct Peer {
    incoming_messages: MessageQueue,
    outgoing_messages: MessageQueue,
    state: PeerState,

    disconnect_timeout: u32,
    disconnect_notify_start: u32,

    round_trip_time: u32,
    kbps_sent: u32,

    local_frame_advantage: Frame,
    remote_frame_advantage: Frame,
}

impl Peer {
    pub fn send_input<T>(&mut self, input: GameInput<T>) {}

    pub fn send_input_ack(&mut self) {}

    pub fn state(&self) -> &PeerState {
        &self.state
    }

    pub fn disconnect(&mut self) {
        self.state = PeerState::Disconnected;
        // _shutdown_timeout = Platform::GetCurrentTimeMS() + UDP_SHUTDOWN_TIMER;
    }

    pub fn set_disconnect_timeout(&mut self, timeout: u32) {
        self.disconnect_timeout = timeout;
    }

    pub fn set_disconnect_notify_start(&mut self, timeout: u32) {
        self.disconnect_notify_start = timeout;
    }

    pub fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            ping: self.round_trip_time,
            send_queue_len: self.outgoing_messages.len(),
            recv_queue_len: self.incoming_messages.len(),
            kbps_sent: self.kbps_sent,

            local_frames_behind: self.local_frame_advantage,
            remote_frames_behind: self.remote_frame_advantage,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionStatus {
    pub disconnected: bool,
    pub last_frame: Frame,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self {
            disconnected: false,
            last_frame: super::NULL_FRAME,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    magic: u16,
    sequence_number: u16,
    data: MessageData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageData {
    KeepAlive,
    SyncRequest {
        random_request: u32,
        remote_magic: u16,
        remote_endpoint: u8,
    },
    SyncReply {
        random_reply: u32,
    },
    Input {
        peer_connect_status: [ConnectionStatus; 8],
        start_frame: u32,

        // Highest bit is a disconnection request.
        // Lower 31 bits are specifying the latest acked frames.
        ack_frame: u32,
        bits: Vec<u8>,
    },
    QualityReport {
        frame_advantage: i8,
        ping: u32,
    },
    QualityReply {
        pong: u32,
    },
    InputAck {
        ack_frame: u32,
    },
}

struct Stats {
    ping: i32,
    remote_frame_advantage: i32,
    local_frame_advantage: i32,
    send_queue_len: usize,
    // Udp::Stats          udp;
}

pub enum Event<T> {
    Connected,
    Synchronizing { total: i32, count: i32 },
    Synchronized,
    Input { input: GameInput<T> },
    Disconnected,
    NetworkInterrupted { disconnect_timeout: i32 },
    NetworkResumed,
}
