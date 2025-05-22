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

/* #region define_cmd */
pub const CMD_CODE_DATA_TRRE: u8 = 0x10;
pub const CMD_CODE_VECH_CONTROL: u8 = 0x20;

pub const CMD_CODE_LOOP_STOP: u8 = 0x00;
pub const CMD_CODE_ONLY_ONCE: u8 = 0x01;
pub const CMD_CODE_LOOP_START: u8 = 0x02;

pub const CMD_CODE_MOTOR_LEFT: u8 = 0x00;
pub const CMD_CODE_MOTOR_RIGHT: u8 = 0x01;

pub const CMD_CODE_SPEED: u8  = 0x00;
pub const CMD_CODE_ADC: u8  = 0x05;

define_cmd!( LEFT_SPEED_STORE, "CMD_LEFT_SPEED_STORE",
    [CMD_CODE_MOTOR_LEFT, CMD_CODE_SPEED]
);
define_cmd!( LEFT_SPEED_STOP, "CMD_LEFT_SPEED_STOP",
    [CMD_CODE_MOTOR_LEFT, CMD_CODE_SPEED, CMD_CODE_LOOP_STOP]
);
define_cmd!( LEFT_SPEED_ONCE, "CMD_LEFT_SPEED_ONCE",
    [CMD_CODE_MOTOR_LEFT, CMD_CODE_SPEED, CMD_CODE_ONLY_ONCE]
);
define_cmd!( LEFT_SPEED_START, "CMD_LEFT_SPEED_START",
    [CMD_CODE_MOTOR_LEFT, CMD_CODE_SPEED, CMD_CODE_LOOP_START]
);
define_cmd!( LEFT_ADC_STORE, "CMD_LEFT_ADC_STORE",
    [CMD_CODE_MOTOR_LEFT, CMD_CODE_ADC]
);
define_cmd!( LEFT_ADC_STOP, "CMD_LEFT_ADC_STOP",
    [CMD_CODE_MOTOR_LEFT, CMD_CODE_ADC, CMD_CODE_LOOP_STOP]
);
define_cmd!( LEFT_ADC_ONCE, "CMD_LEFT_ADC_ONCE",
    [CMD_CODE_MOTOR_LEFT, CMD_CODE_ADC, CMD_CODE_ONLY_ONCE]
);
define_cmd!( LEFT_ADC_START, "CMD_LEFT_ADC_START",
    [CMD_CODE_MOTOR_LEFT, CMD_CODE_ADC, CMD_CODE_LOOP_START]
);

define_cmd!( RIGHT_SPEED_STORE, "CMD_RIGHT_SPEED_STORE",
    [CMD_CODE_MOTOR_RIGHT, CMD_CODE_SPEED]
);
define_cmd!( RIGHT_SPEED_STOP, "CMD_RIGHT_SPEED_STOP",
    [CMD_CODE_MOTOR_RIGHT, CMD_CODE_SPEED, CMD_CODE_LOOP_STOP]
);
define_cmd!( RIGHT_SPEED_ONCE, "CMD_RIGHT_SPEED_ONCE",
    [CMD_CODE_MOTOR_RIGHT, CMD_CODE_SPEED, CMD_CODE_ONLY_ONCE]
);
define_cmd!( RIGHT_SPEED_START, "CMD_RIGHT_SPEED_START",
    [CMD_CODE_MOTOR_RIGHT, CMD_CODE_SPEED, CMD_CODE_LOOP_START]
);
define_cmd!( RIGHT_ADC_STORE, "CMD_RIGHT_ADC_STORE",
    [CMD_CODE_MOTOR_RIGHT, CMD_CODE_ADC]
);
define_cmd!( RIGHT_ADC_STOP, "CMD_RIGHT_ADC_STOP",
    [CMD_CODE_MOTOR_RIGHT, CMD_CODE_ADC, CMD_CODE_LOOP_STOP]
);
define_cmd!( RIGHT_ADC_ONCE, "CMD_RIGHT_ADC_ONCE",
    [CMD_CODE_MOTOR_RIGHT, CMD_CODE_ADC, CMD_CODE_ONLY_ONCE]
);
define_cmd!( RIGHT_ADC_START, "CMD_RIGHT_ADC_START",
    [CMD_CODE_MOTOR_RIGHT, CMD_CODE_ADC, CMD_CODE_LOOP_START]
);

define_cmd!(
    STOP,      "CMD_MOVE_STOP",
    [CMD_CODE_VECH_CONTROL, 0x00]
);
define_cmd!(
    FORWARD,   "CMD_MOVE_FORWARD",
    [CMD_CODE_VECH_CONTROL, 0x01]
);
define_cmd!(
    BACKWARD,  "CMD_MOVE_BACKWARD",
    [CMD_CODE_VECH_CONTROL, 0x02]
);
define_cmd!(
    LEFT,      "CMD_MOVE_LEFT",
    [CMD_CODE_VECH_CONTROL, 0x03]
);
define_cmd!(
    RIGHT,     "CMD_MOVE_RIGHT",
    [CMD_CODE_VECH_CONTROL, 0x04]
);
/* #endregion */
