use log::error;
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

const CODE_DATA_TRRE: u8 = 0x10;
const CODE_VECH_CONTROL: u8 = 0x20;
const CODE_LOOP_STOP: u8 = 0x00;
const CODE_ONLY_ONCE: u8 = 0x01;
const CODE_LOOP_START: u8 = 0x02;
const CODE_SPEED: u8  = 0x20;
const CODE_ADC: u8  = 0x20;
const CODE_MOTOR_LEFT: u8 = 0x00;
const CODE_MOTOR_RIGHT: u8 = 0x01;

/* #region define_cmd */
define_cmd!( LEFT_SPEED_STORE, "PC_CMD_LEFT_SPEED_STORE",
    [CODE_MOTOR_LEFT, CODE_SPEED]
);
define_cmd!( LEFT_SPEED_STOP, "PC_CMD_LEFT_SPEED_STOP",
    [CODE_DATA_TRRE, CODE_MOTOR_LEFT, CODE_SPEED, CODE_LOOP_STOP]
);
define_cmd!( LEFT_SPEED_ONCE, "PC_CMD_LEFT_SPEED_ONCE",
    [CODE_DATA_TRRE, CODE_MOTOR_LEFT, CODE_SPEED, CODE_ONLY_ONCE]
);
define_cmd!( LEFT_SPEED_START, "PC_CMD_LEFT_SPEED_START",
    [CODE_DATA_TRRE, CODE_MOTOR_LEFT, CODE_SPEED, CODE_LOOP_START]
);
define_cmd!( LEFT_ADC_STORE, "PC_CMD_LEFT_ADC_STORE",
    [CODE_MOTOR_LEFT, CODE_ADC]
);
define_cmd!( LEFT_ADC_STOP, "PC_CMD_LEFT_ADC_STOP",
    [CODE_DATA_TRRE, CODE_MOTOR_LEFT, CODE_ADC, CODE_LOOP_STOP]
);
define_cmd!( LEFT_ADC_ONCE, "PC_CMD_LEFT_ADC_ONCE",
    [CODE_DATA_TRRE, CODE_MOTOR_LEFT, CODE_ADC, CODE_ONLY_ONCE]
);
define_cmd!( LEFT_ADC_START, "PC_CMD_LEFT_ADC_START",
    [CODE_DATA_TRRE, CODE_MOTOR_LEFT, CODE_ADC, CODE_LOOP_START]
);

define_cmd!( RIGHT_SPEED_STORE, "PC_CMD_RIGHT_SPEED_STORE",
    [CODE_MOTOR_RIGHT, CODE_SPEED]
);
define_cmd!( RIGHT_SPEED_STOP, "PC_CMD_RIGHT_SPEED_STOP",
    [CODE_DATA_TRRE, CODE_MOTOR_RIGHT, CODE_SPEED, CODE_LOOP_STOP]
);
define_cmd!( RIGHT_SPEED_ONCE, "PC_CMD_RIGHT_SPEED_ONCE",
    [CODE_DATA_TRRE, CODE_MOTOR_RIGHT, CODE_SPEED, CODE_ONLY_ONCE]
);
define_cmd!( RIGHT_SPEED_START, "PC_CMD_RIGHT_SPEED_START",
    [CODE_DATA_TRRE, CODE_MOTOR_RIGHT, CODE_SPEED, CODE_LOOP_START]
);
define_cmd!( RIGHT_ADC_STORE, "PC_CMD_RIGHT_ADC_STORE",
    [CODE_MOTOR_RIGHT, CODE_ADC]
);
define_cmd!( RIGHT_ADC_STOP, "PC_CMD_RIGHT_ADC_STOP",
    [CODE_DATA_TRRE, CODE_MOTOR_RIGHT, CODE_ADC, CODE_LOOP_STOP]
);
define_cmd!( RIGHT_ADC_ONCE, "PC_CMD_RIGHT_ADC_ONCE",
    [CODE_DATA_TRRE, CODE_MOTOR_RIGHT, CODE_ADC, CODE_ONLY_ONCE]
);
define_cmd!( RIGHT_ADC_START, "PC_CMD_RIGHT_ADC_START",
    [CODE_DATA_TRRE, CODE_MOTOR_RIGHT, CODE_ADC, CODE_LOOP_START]
);

define_cmd!(
    STOP,      "PC_CMD_MOVE_STOP",
    [CODE_VECH_CONTROL, 0x00]
);
define_cmd!(
    FORWARD,   "PC_CMD_MOVE_FORWARD",
    [CODE_VECH_CONTROL, 0x01]
);
define_cmd!(
    BACKWARD,  "PC_CMD_MOVE_BACKWARD",
    [CODE_VECH_CONTROL, 0x02]
);
define_cmd!(
    LEFT,      "PC_CMD_MOVE_LEFT",
    [CODE_VECH_CONTROL, 0x03]
);
define_cmd!(
    RIGHT,     "PC_CMD_MOVE_RIGHT",
    [CODE_VECH_CONTROL, 0x04]
);
/* #endregion */

pub struct DataStore {
    left_speed: Vec<f32>,
    right_speed: Vec<f32>,
    left_adc: Vec<u16>,
    right_adc: Vec<u16>,
}
pub enum DataStoreSelF32 {
    LeftSpeed,
    RightSpeed,
}
pub enum DataStoreSelU16 {
    LeftAdc,
    RightAdc,
}
macro_rules! push_then_truncate {
    ($vec:expr, $val:expr, $max:expr) => {{
        $vec.push($val);
        if $vec.len() > $max {
            $vec.remove(0);
        }
    }};
}
impl DataStore {
    pub fn new() -> Self {
        Self {
            left_speed: Vec::new(),
            right_speed: Vec::new(),
            left_adc: Vec::new(),
            right_adc: Vec::new(),
        }
    }

    pub async fn push_f32(&mut self, select: DataStoreSelF32, value: f32) {
        match select {
            DataStoreSelF32::LeftSpeed =>    push_then_truncate!(self.left_speed, value, 100),
            DataStoreSelF32::RightSpeed =>   push_then_truncate!(self.right_speed, value, 100),
        }
    }
    pub async fn push_i16(&mut self, select: DataStoreSelU16, value: u16) {
        match select {
            DataStoreSelU16::LeftAdc =>      push_then_truncate!(self.left_adc, value, 100),
            DataStoreSelU16::RightAdc =>     push_then_truncate!(self.right_adc, value, 100),
        }
    }

    pub async fn read_f32(&mut self, select: DataStoreSelF32) -> &[f32] {
        match select {
            DataStoreSelF32::LeftSpeed => &self.left_speed,
            DataStoreSelF32::RightSpeed => &self.right_speed,
        }
    }
    pub async fn read_u16(&mut self, select: DataStoreSelU16) -> &[u16] {
        match select {
            DataStoreSelU16::LeftAdc => &self.left_adc,
            DataStoreSelU16::RightAdc => &self.right_adc,
        }
    }
}

pub async fn send_cmd(app: AppHandle, cmd: &[u8]) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
    let packet = UartPacket::new(cmd.to_vec())?;
    transfer_buffer.push(packet)
}

pub async fn re_pkt_proccess(app: AppHandle) {
    let global_state = app.state::<GlobalState>();
    for _ in 0..10 {
        let maybe_pkt = {
            let mut receive_buffer = global_state.receive_buffer.lock().await;
            receive_buffer.pop_front()
        };
        let mut data = match maybe_pkt {
            None => break,
            Some(p) => p.data(),
        };
        match data.remove(0) {
            cmd if cmd == CODE_DATA_TRRE => re_pkt_data_store(app.clone(), data).await,
            cmd if cmd == CODE_VECH_CONTROL => break,
            _ => break,
        };
    }
}

async fn re_pkt_data_store(app: AppHandle, mut data: Vec<u8>) {
    let global_state = app.state::<GlobalState>();
    loop {
        if        data.starts_with(LEFT_SPEED_STORE.payload) {
            data.drain(..LEFT_SPEED_STORE.payload.len());
            let value = match data[..size_of::<f32>()].try_into() {
                Ok(bytes) => {
                    data.drain(..size_of::<f32>());
                    f32::from_bits(u32::from_be_bytes(bytes))
                },
                Err(e) => {
                    error!("ERR: {}", e);
                    break;
                }
            };
            let mut store_datas = global_state.store_datas.lock().await;
            store_datas.push_f32(DataStoreSelF32::LeftSpeed, value).await;
        }
        else if data.starts_with(RIGHT_SPEED_STORE.payload) {
            data.drain(..RIGHT_SPEED_STORE.payload.len());
            let value = match data[..size_of::<f32>()].try_into() {
                Ok(bytes) => {
                    data.drain(..size_of::<f32>());
                    f32::from_bits(u32::from_be_bytes(bytes))
                },
                Err(e) => {
                    error!("ERR: {}", e);
                    break;
                }
            };
            let mut store_datas = global_state.store_datas.lock().await;
            store_datas.push_f32(DataStoreSelF32::RightSpeed, value).await;
        }
        else if data.starts_with(LEFT_ADC_STORE.payload) {
            data.drain(..LEFT_ADC_STORE.payload.len());
            let value = match data[..size_of::<u16>()].try_into() {
                Ok(bytes) => {
                    data.drain(..size_of::<u16>());
                    u16::from_be_bytes(bytes)
                },
                Err(e) => {
                    error!("ERR: {}", e);
                    break;
                }
            };
            let mut store_datas = global_state.store_datas.lock().await;
            store_datas.push_i16(DataStoreSelU16::LeftAdc, value).await;
        }
        else if data.starts_with(RIGHT_ADC_STORE.payload) {
            data.drain(..RIGHT_ADC_STORE.payload.len());
            let value = match data[..size_of::<u16>()].try_into() {
                Ok(bytes) => {
                    data.drain(..size_of::<u16>());
                    u16::from_be_bytes(bytes)
                },
                Err(e) => {
                    error!("ERR: {}", e);
                    break;
                }
            };
            let mut store_datas = global_state.store_datas.lock().await;
            store_datas.push_i16(DataStoreSelU16::RightAdc, value).await;
        }
        else { break; }
    }
}
