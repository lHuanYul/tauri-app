#[derive(Debug)]
pub enum DataSlice<'a> {
    U8(&'a [u8]),
    U16(&'a [u16]),
    F32(&'a [f32]),
}

pub struct MotorDataStore {
    max_length: usize,
    adc_value: Vec<u16>,
    speed_setpoint: Vec<u8>,
    speed_present: Vec<f32>,
    rotate_direction: Vec<u8>,
}
pub enum MotorDataType {
    AdcValue,
    SpeedSetpoint,
    SpeedPresent,
    RotateDirection,
}
impl MotorDataStore {
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            adc_value: Vec::new(),
            speed_setpoint: Vec::new(),
            speed_present: Vec::new(),
            rotate_direction: Vec::new(),
        }
    }

    pub fn push(&mut self, kind: MotorDataType, value: impl Into<u64>) {
        match kind {
            MotorDataType::AdcValue => {
                let v = value.into() as u16;
                let overflow = self.adc_value.len().saturating_sub(self.max_length);
                self.adc_value.drain(0..overflow);
                self.adc_value.push(v);
            }
            MotorDataType::SpeedSetpoint => {
                let v = value.into() as u8;
                let overflow = self.speed_setpoint.len().saturating_sub(self.max_length);
                self.speed_setpoint.drain(0..overflow);
                self.speed_setpoint.push(v);
            }
            MotorDataType::SpeedPresent => {
                let v = value.into() as f32;
                let overflow = self.speed_present.len().saturating_sub(self.max_length);
                self.speed_present.drain(0..overflow);
                self.speed_present.push(v);
            }
            MotorDataType::RotateDirection => {
                let v = value.into() as u8;
                let overflow = self.rotate_direction.len().saturating_sub(self.max_length);
                self.rotate_direction.drain(0..overflow);
                self.rotate_direction.push(v);
            }
        }
    }

    pub fn get(&self, kind: MotorDataType) -> DataSlice<'_> {
        match kind {
            MotorDataType::AdcValue => DataSlice::U16(&self.adc_value),
            MotorDataType::SpeedSetpoint => DataSlice::U8(&self.speed_setpoint),
            MotorDataType::SpeedPresent => DataSlice::F32(&self.speed_present),
            MotorDataType::RotateDirection => DataSlice::U8(&self.rotate_direction),
        }
    }

    pub fn clear(&mut self, kind: MotorDataType) {
        match kind {
            MotorDataType::AdcValue => self.adc_value.clear(),
            MotorDataType::SpeedSetpoint => self.speed_setpoint.clear(),
            MotorDataType::SpeedPresent => self.speed_present.clear(),
            MotorDataType::RotateDirection => self.rotate_direction.clear(),
        }
    }
}

pub struct DataStore {
    motor_left: MotorDataStore,
    motor_right: MotorDataStore,
    vehicel_pos: Vec<u16>,
}
pub enum DataType {
    MotorLeft(MotorDataType),
    MotorRight(MotorDataType),
    VehicelPos,
}
impl DataStore {
    pub fn new(max_length: usize) -> Self {
        Self {
            motor_left:  MotorDataStore::new(max_length),
            motor_right: MotorDataStore::new(max_length),
            vehicel_pos: Vec::new(),
        }
    }

    pub fn push(&mut self, dt: DataType, value: impl Into<u64>) {
        match dt {
            DataType::MotorLeft(kind) => {
                self.motor_left.push(kind, value);
            }
            DataType::MotorRight(kind) => {
                self.motor_right.push(kind, value);
            }
            DataType::VehicelPos => {
                let v = value.into() as u16;
                self.vehicel_pos.push(v);
            }
        }
    }

    pub fn get(&self, dt: DataType) -> DataSlice<'_> {
        match dt {
            DataType::MotorLeft(kind) => self.motor_left.get(kind),
            DataType::MotorRight(kind) => self.motor_right.get(kind),
            DataType::VehicelPos => {
                // 如果 DataSlice 目前沒法承載 u16 slice，你可以改回傳 &[u16]
                DataSlice::U16(&self.vehicel_pos)
            }
        }
    }

    pub fn clear(&mut self, dt: DataType) {
        match dt {
            DataType::MotorLeft(kind) => self.motor_left.clear(kind),
            DataType::MotorRight(kind) => self.motor_right.clear(kind),
            DataType::VehicelPos => self.vehicel_pos.clear(),
        }
    }
}