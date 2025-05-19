use log::{error, info};
use tauri::{AppHandle, Manager};
use crate::{mods::packet_mod::UartPacket, GlobalState};

pub struct CmdInfo {
    pub name:       &'static str,
    pub payload:    &'static [u8],
}
macro_rules! define_cmd {
    ($const_name:ident, $name_str:expr, [$($bytes:expr),+ $(,)?]) => {
        pub const $const_name: CmdInfo = CmdInfo {
            name: $name_str,
            payload: &[$($bytes),+],
        };
    };
}

pub struct Send;
impl Send {
    const PREFIX_CODE: u8 = 0x10;

    const SPEED_CODE: u8  = 0x20;
    define_cmd!(
        SPEED_STOP,     "PC_CMD_SEND_SPEED_STOP",
        [Self::PREFIX_CODE, Self::SPEED_CODE, 0x00]
    );
    define_cmd!(
        SPEED_ONCE,     "PC_CMD_SEND_SPEED_ONCE",
        [Self::PREFIX_CODE, Self::SPEED_CODE, 0x01]
    );
    define_cmd!(
        SPEED_START,    "PC_CMD_SEND_SPEED_START",
        [Self::PREFIX_CODE, Self::SPEED_CODE, 0x02]
    );
}

pub struct Receive;
impl Receive {
    const PREFIX_CODE:   u8 = 0x10;

    define_cmd!(
        SPEED,     "PC_CMD_RECEIVE_SPEED",
        [0x20]
    );
    define_cmd!(
        SPEED_2,     "PC_CMD_RECEIVE_SPEED",
        [Self::PREFIX_CODE, 0x21]
    );
    
}

pub struct Movement;
impl Movement {
    const PREFIX:   u8   = 0x20;
    define_cmd!(
        STOP,      "PC_CMD_MOVE_STOP",
        [Self::PREFIX, 0x00]
    );
    define_cmd!(
        FORWARD,   "PC_CMD_MOVE_FORWARD",
        [Self::PREFIX, 0x01]
    );
    define_cmd!(
        BACKWARD,  "PC_CMD_MOVE_BACKWARD",
        [Self::PREFIX, 0x02]
    );
    define_cmd!(
        LEFT,      "PC_CMD_MOVE_LEFT",
        [Self::PREFIX, 0x03]
    );
    define_cmd!(
        RIGHT,     "PC_CMD_MOVE_RIGHT",
        [Self::PREFIX, 0x04]
    );
}

pub async fn send_cmd(app: AppHandle, cmd: &[u8]) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
    let packet = UartPacket::new(cmd.to_vec())?;
    transfer_buffer.push(packet)
}

pub async fn receive_packet_proccess(app: AppHandle) {
    let global_state = app.state::<GlobalState>();
    for _ in 0..10 {
        let maybe_pkt = {
            let mut receive_buffer = global_state.receive_buffer.lock().await;
            receive_buffer.pop_front()
        };
        let data = match maybe_pkt {
            None => break,
            Some(p) => p.data(),
        };
        info!("write_speed {:?}", data);
        if let Some((&cmd, payload)) = data.split_first() {
            let _ = match cmd {
                x if x == Receive::SPEED.payload[0] => write_speed(app.clone(), payload).await,
                x if x == Receive::SPEED_2.payload[0] => write_speed(app.clone(), payload).await,
                _ => break,
            };
        }
    }
}

pub async fn write_speed(app: AppHandle, data: &[u8]) ->Result<(), String> {
    if data.len() < 4 {
        let message = format!("Speed 資料長度不足 只有 {} bytes 無法解析 f32", data.len());
        error!("{}", message);
        return Err(message)
    }
    let bytes: [u8; 4] = data[0..4].try_into().unwrap();
    let value = u32::from_be_bytes(bytes);
    let global_state = app.state::<GlobalState>();
    let mut chart_state = global_state.speed_datas.lock().await;
    chart_state.push(value);
    if data.len() > 4 {
        error!(
            "Speed 資料有多餘 {} 個 byte 已忽略",
            data.len() - 4
        );
    }
    Ok(())
}
