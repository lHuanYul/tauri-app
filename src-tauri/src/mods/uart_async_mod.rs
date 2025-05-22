use std::{sync::Arc, time::Duration};
use log::{debug, error, info, trace};
use tauri::{AppHandle, Manager};
use serialport::{available_ports, SerialPortInfo};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf}, sync::{watch::{channel, Receiver, Sender}, Mutex}, time::{sleep, timeout}};
use crate::{mods::packet_mod::{UartPacket, PACKET_END_CODE, PACKET_MAX_SIZE}, GlobalState};
use super::{log_mod::CODE_TRACE, mcu_control_mod::{self}};

/// 非同步讀取單一位元組的超時值（µs）<br>
/// Default timeout for each byte read in µs
const   PORT_READ_TIMEOUT_US:       u64     = 1000;
/// 最大接收緩衝區大小（包含起始與結尾碼）<br>
/// Maximum receive buffer size (including start and end codes)
const   MAX_RECEIVE_BUFFER_SIZE:    usize   = PACKET_MAX_SIZE;

/// 非同步序列埠管理器 <br>
/// Asynchronous serial port manager
pub struct PortAsyncManager {
    port_name: Option<String>,          // 序列埠名稱／port name
    inner: Arc<PortAsyncManagerInner>,  // 內部管理結構／inner manager
    shutdown_tx: Option<Sender<bool>>,  // 停止訊號傳送者／shutdown signal sender
}
impl PortAsyncManager {
    /// 建立新管理器，尚未開啟任何埠 <br>
    /// Creates a new manager with no open port
    pub fn new() -> Self {
        Self {
            port_name: None,
            inner: Arc::new(PortAsyncManagerInner::new()),
            shutdown_tx: None,
        }
    }

    /// 列出所有可用序列埠資訊 <br>
    /// Lists all available serial port infos asynchronously
    pub async fn available() -> Result<Vec<SerialPortInfo>, String> {
        let ports = available_ports().map_err(|e| {format!("Get available ports failed: {}", e)})?;
        Ok(ports)
    }

    /// 開啟指定序列埠並啟動讀寫迴圈 <br>
    /// Opens the specified port and starts the read/write loop
    pub async fn open(
        &mut self,
        app: AppHandle,
        port_name: &str,
        baudrate: u32,
        timeout_ms: u64,
    ) -> Result<(), String> {
        let stream = tokio_serial::new(port_name, baudrate)
            .timeout(Duration::from_millis(timeout_ms))
            .open_native_async()
            .map_err(|e| format!("Port open failed: {}", e))?;
        self.port_name = Some(port_name.to_string());
        let (reader, writer) = tokio::io::split(stream);
        *self.inner.reader.lock().await = Some(reader);
        *self.inner.writer.lock().await = Some(writer);
        let (shutdown_tx, shutdown_rx) = channel(false);
        self.shutdown_tx = Some(shutdown_tx.clone());

        self.inner.spawn_read_write_loop(app, shutdown_rx);
        Ok(())
    }

    /// 關閉目前序列埠，並停止讀寫迴圈 <br>
    /// Closes the current port and stops the read/write loop
    pub async fn close(&mut self) -> Result<(), String> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(true);
        }
        self.port_name = None;
        *self.inner.reader.lock().await = None;
        *self.inner.writer.lock().await = None;
        self.shutdown_tx = None;
        Ok(())
    }

    /// 檢查序列埠是否已開啟<br>
    /// Checks whether the port is currently open
    pub async fn check_open(&self) -> Result<(), String> {
        self.inner.check_open().await
    }
}

/// 內部序列埠管理結構 <br>
/// Internal struct for managing serial port operations
pub struct PortAsyncManagerInner {
    reader: Mutex<Option<ReadHalf<SerialStream>>>,  // 讀取半部／read half
    writer: Mutex<Option<WriteHalf<SerialStream>>>, // 寫入半部／write half
}
impl PortAsyncManagerInner {
    /// 建立內部管理實例 <br>
    /// Creates the inner manager instance
    fn new() -> Self {
        Self {
            reader: None.into(),
            writer: None.into(),
        }
    }

    /// 檢查埠是否開啟，否則回傳錯誤 <br>
    /// Checks if the port is open, returns error if not
    async fn check_open(&self) -> Result<(), String> {
        if self.reader.lock().await.is_none() || self.writer.lock().await.is_none() {
            return Err("Port is not open".into());
        }
        Ok(())
    }

    /// 非同步讀取並解析完整封包 <br>
    /// Asynchronously reads and parses one full UartPacket
    async fn read_packet(&self) -> Result<UartPacket, String> {
        self.check_open().await?;
        let mut reader_guard = self.reader.lock().await;
        let reader = reader_guard.as_mut().unwrap();

        let mut buffer: Vec<u8> = Vec::with_capacity(MAX_RECEIVE_BUFFER_SIZE);
        for _ in 0..MAX_RECEIVE_BUFFER_SIZE {
            match timeout(
                Duration::from_micros(PORT_READ_TIMEOUT_US),
                reader.read_u8(),
            ).await {
                Ok(Ok(byte)) => {
                    buffer.push(byte);
                },
                Ok(Err(e)) => { return Err(format!("Read u8 failed: {}", e)); }
                Err(_) => {
                    if buffer.is_empty() { return Err(format!("{}Read nothing", CODE_TRACE)); }
                    if *buffer.last().unwrap() != PACKET_END_CODE {
                        return Err(format!(
                            "Read timeout at {} bytes (or no end code)\n>>> {:?}",
                            buffer.len(), buffer
                        ));
                    }
                    break;
                }
            };
        }
        
        let packet = UartPacket::pack(buffer)?;
        Ok(packet)
    }

    /// 非同步寫入封包到序列埠 <br>
    /// Asynchronously writes a UartPacket to the serial port
    async fn write_packet(&self, packet: UartPacket) -> Result<(), String> {
        self.check_open().await?;
        let mut writer_guard = self.writer.lock().await;
        let writer = writer_guard.as_mut().unwrap();
        let buffer = packet.unpack()?;
        writer.write_all(&buffer).await.map_err(|e| format!("Write failed: {}", e))?;
        Ok(())
    }

    /// 建立並啟動讀寫背景工作迴圈 <br>
    /// Spawns background tasks for continuous read/write loops
    fn spawn_read_write_loop(
        self: &Arc<Self>,
        app: AppHandle,
        shutdown_rx: Receiver<bool>,
    ) {
        let arc_read = Arc::clone(self);
        let shutdown_read = shutdown_rx.clone();
        let read_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown_read.borrow() { break; }
                let result = arc_read.read_packet().await;
                if let Err(e) = &result {
                    if let Some(rest) = e.strip_prefix(CODE_TRACE) {
                        trace!("{}", rest);
                    } else {
                        error!("{}", e);
                    }
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
                let packet = result.unwrap();
                debug!("Port read succeed:\n{}", packet.show());
                {
                    let global_state = read_handle.state::<GlobalState>();
                    let mut receive_buffer = global_state.receive_buffer.lock().await;
                    let _ = receive_buffer.push(packet).map_err(|e| {
                        error!("Packet store failed: {}", e);
                    });
                    receive_buffer.show(5);
                };
            }
        });

        let arc_write = Arc::clone(self);
        let shutdown_write = shutdown_rx.clone();
        let write_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown_write.borrow() { break; }
                let maybe_pkt = {
                    let global_state = write_handle.state::<GlobalState>();
                    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
                    transfer_buffer.pop_front()
                };
                if maybe_pkt.is_none() {
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
                let packet = maybe_pkt.unwrap();
                let _ = arc_write.write_packet(packet.clone()).await.map_err(|e| {
                    error!("Port write failed: {}", e);
                });
                debug!("Port write succeed:\n{}", packet.show());
            }
        });
    }
}

/// 列出可用序列埠名稱 <br>
/// Tauri command: list available port names
#[tauri::command]
pub async fn cmd_available_port_async() -> Result<Vec<String>, String> {
    let ports = PortAsyncManager::available().await?;
    let names = ports.into_iter().rev().map(|info| info.port_name).collect();
    info!("All available ports: {:?}", names);
    Ok(names)
}

/// 檢查序列埠是否已開啟 <br>
/// Tauri command: check if port is open
#[tauri::command]
pub async fn cmd_check_port_open_async(app: AppHandle) -> bool {
    let global_state = app.state::<GlobalState>();
    let state = global_state.main_port.lock().await;
    state.check_open().await.is_ok()
}

/// 開啟指定序列埠 <br>
/// Tauri command: open specified port
#[tauri::command]
pub async fn cmd_open_port_async(app: AppHandle, port_name: String) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let mut state = global_state.main_port.lock().await;
    state.open(app.clone(), &port_name, 115200, 1000).await.map_err(|e| {
        error!("{}", e);
        e.clone()
    })?;
    let _msg = format!("Open port succeed: {}", port_name);
    info!("{}", _msg);
    Ok(_msg)
}

/// 關閉目前序列埠 <br>
/// Tauri command: close current port
#[tauri::command]
pub async fn cmd_close_port_async(app: AppHandle) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let mut port = global_state.main_port.lock().await;
    port.close().await.map_err(|e| {
        error!("{}", e);
        e.clone()
    })?;
    let message = "Close port succeed".into();
    info!("{}", message);
    Ok(message)
}

/// 測試封包寫入與讀取 <br>
/// Tauri command: test packet write and read
#[tauri::command]
pub async fn cmd_serial_test(app: AppHandle) -> Result<String, String> {
    mcu_control_mod::send_cmd(app, mcu_control_mod::RIGHT_SPEED_ONCE.payload).await.map_err(|e| {
        let message = format!("Send failed: {}", e);
        error!("{}", message);
        message
    })?;
    Ok("Push finish".into())
}
