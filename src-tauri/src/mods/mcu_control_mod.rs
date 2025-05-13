use tauri::{AppHandle, Manager};
use crate::{mods::packet_mod::UartPacket, GlobalState};

pub struct Movement;
impl Movement {
    const PREFIX: u8 = 0x10;
    /// 停止命令編碼<br>
    /// Command code for stop
    pub const STOP:         &[u8] = &[Self::PREFIX, 0x00];
    /// 前進命令編碼<br>
    /// Command code for moving forward
    pub const FORWARD:      &[u8] = &[Self::PREFIX, 0x01];
    /// 後退命令編碼<br>
    /// Command code for moving backward
    pub const BACKWARD:     &[u8] = &[Self::PREFIX, 0x02];
    /// 左轉命令編碼<br>
    /// Command code for turning left
    pub const TURN_LEFT:    &[u8] = &[Self::PREFIX, 0x03];
    /// 右轉命令編碼<br>
    /// Command code for turning right
    pub const TURN_RIGHT:   &[u8] = &[Self::PREFIX, 0x04];
}

pub struct Send;
impl Send {
    const PREFIX: u8 = 0x80;

    pub const SPEED:    &[u8] = &[Self::PREFIX, 0x00];
}

pub async fn send_cmd(app: AppHandle, cmd: &[u8]) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
    let packet = UartPacket::new(cmd.to_vec())?;
    transfer_buffer.push(packet)
}
