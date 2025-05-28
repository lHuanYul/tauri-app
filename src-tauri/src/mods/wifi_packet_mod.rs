use std::{error::Error, net::IpAddr};

use crate::mods::user_vec_mod::UserVecU8;

pub const WIFI_TCP_PACKET_MAX_SIZE: usize = 1024;
pub const WIFI_UDP_PACKET_MAX_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct WifiPacket {
    target_ip: IpAddr,
    data: UserVecU8,
}
impl WifiPacket {
    pub fn new<T: AsRef<[u8]>>(target_ip: IpAddr, data: T) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut user_vec = UserVecU8::new(WIFI_TCP_PACKET_MAX_SIZE);
        user_vec.extend(data)?;
        Ok( Self {
            target_ip,
            data: user_vec,
        })
    }

    pub fn target_ip(&self) -> IpAddr { self.target_ip.clone() }

    /// 取得封包資料參考<br>
    /// Returns a reference to the packet data array
    pub fn data(&self) -> Vec<u8> { self.data.show() }
    
    /// 顯示封包內容為字串<br>
    /// Formats packet contents into a string for display
    pub fn show(&self) -> String {
        format!(
            ">>>   {:?}",
            self.data.show(),
        )
    }

    // pub fn pack(data: Vec<u8>) -> Result<Self, Box<dyn Error + Send + Sync>> {
    //     if data.first() != Some(&UART_PACKET_START_CODE) {
    //         let message = format!("Start byte invalid (expected {:?}): {:?}", UART_PACKET_START_CODE, data.first());
    //         return Err(message.into());
    //     }
    //     if data.last() != Some(&UART_PACKET_END_CODE) {
    //         let message = format!("End byte invalid (expected {:?}): {:?}", UART_PACKET_END_CODE, data.last());
    //         return Err(message.into());
    //     }
    //     let packet = Self::new(data[1..(data.len()-1)].to_vec())?;
    //     Ok(packet)
    // }

    // pub fn unpack(&self) -> Vec<u8> {
    //     let mut buffer= Vec::<u8>::new();
    //     buffer.push(self.start);
    //     buffer.extend_from_slice(&self.data.show());
    //     buffer.push(self.end);
    //     buffer
    // }
}
