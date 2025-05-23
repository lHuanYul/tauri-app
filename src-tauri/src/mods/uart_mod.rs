use std::{sync::Arc, time::Duration};
use log::{debug, error, info, trace};
use tauri::{AppHandle, Manager};
use serialport::{available_ports, SerialPortInfo};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf}, sync::{watch::{channel, Receiver, Sender}, Mutex}, time::{sleep, timeout}};
use crate::{mods::{log_mod::CODE_TRACE, packet_mod::{UserPacket, PACKET_END_CODE, PACKET_MAX_SIZE}}, GlobalState};

/// 非同步讀取單一位元組的超時值（µs）<br>
/// Default timeout for each byte read in µs
const   PORT_READ_TIMEOUT_US:       u64     = 1000;
/// 最大接收緩衝區大小（包含起始與結尾碼）<br>
/// Maximum receive buffer size (including start and end codes)
const   MAX_RECEIVE_BUFFER_SIZE:    usize   = PACKET_MAX_SIZE;

/// 非同步序列埠管理器 <br>
/// Asynchronous serial port manager
pub struct UartAsyncManager {
    port_name: Option<String>,          // 序列埠名稱／port name
    inner: Arc<UartAsyncManagerInner>,  // 內部管理結構／inner manager
    shutdown: Option<Sender<bool>>,  // 停止訊號傳送者／shutdown signal sender
}
impl UartAsyncManager {
    /// 建立新管理器，尚未開啟任何埠 <br>
    /// Creates a new manager with no open port
    pub fn new() -> Self {
        Self {
            port_name: None,
            inner: Arc::new(UartAsyncManagerInner::new()),
            shutdown: None,
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
        self.shutdown = Some(shutdown_tx.clone());

        self.inner.read_start(app.clone(), shutdown_rx.clone());
        self.inner.write_start(app.clone(), shutdown_rx);
        Ok(())
    }

    /// 關閉目前序列埠，並停止讀寫迴圈 <br>
    /// Closes the current port and stops the read/write loop
    pub async fn close(&mut self) -> Result<(), String> {
        if let Some(shutdown_tx) = self.shutdown.take() {
            let _ = shutdown_tx.send(true);
        }
        self.port_name = None;
        *self.inner.reader.lock().await = None;
        *self.inner.writer.lock().await = None;
        self.shutdown = None;
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
pub struct UartAsyncManagerInner {
    reader: Mutex<Option<ReadHalf<SerialStream>>>,  // 讀取半部／read half
    writer: Mutex<Option<WriteHalf<SerialStream>>>, // 寫入半部／write half
}
impl UartAsyncManagerInner {
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
    async fn read_packet(&self) -> Result<UserPacket, String> {
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
                Ok(Err(e)) => {
                    return Err(format!("Read u8 failed: {}", e));
                }
                Err(_) => {
                    if buffer.is_empty() {
                        return Err(format!("{}Read nothing", CODE_TRACE));
                    }
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
        
        let packet = UserPacket::pack(buffer).map_err(|e| {
            e.to_string()
        })?;
        Ok(packet)
    }

    /// 非同步寫入封包到序列埠 <br>
    /// Asynchronously writes a UartPacket to the serial port
    async fn write_packet(&self, packet: UserPacket) -> Result<(), String> {
        self.check_open().await?;
        let mut writer_guard = self.writer.lock().await;
        let writer = writer_guard.as_mut().unwrap();
        let buffer = packet.unpack().map_err(|e| e.to_string())?;
        writer.write_all(&buffer).await.map_err(|e| format!("Write failed: {}", e))?;
        Ok(())
    }

    pub fn read_start(self: &Arc<Self>, app: AppHandle, shutdown: Receiver<bool>) {
        let arc_handle = Arc::clone(self);
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                // 讀到 true 就 break
                if *shutdown.borrow() { break; }
                match arc_handle.read_packet().await {
                    Err(e) => {
                        let msg = e.to_string();
                        if let Some(rest) = msg.strip_prefix(CODE_TRACE) {
                            trace!("{}", rest);
                        } else {
                            error!("{}", msg);
                        }
                        sleep(Duration::from_millis(10)).await;
                    }
                    Ok(packet) => {
                        debug!("Port read succeed:\n{}", packet.show());
                        let global_state = app_handle.state::<GlobalState>();
                        let mut buf = global_state.uart_recv_buffer.lock().await;
                        if let Err(e) = buf.push(packet) {
                            error!("Packet store failed: {}", e);
                        }
                    }
                }
            }
        });
    }

    pub fn write_start(self: &Arc<Self>, app: AppHandle, shutdown: Receiver<bool>) {
        let arc_handle = Arc::clone(self);
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown.borrow() { break; }
                let maybe_pkt = {
                    let state = app_handle.state::<GlobalState>();
                    let mut buf = state.uart_traf_buffer.lock().await;
                    buf.pop_front()
                };
                if let Some(packet) = maybe_pkt {
                    if let Err(e) = arc_handle.write_packet(packet.clone()).await {
                        error!("Port write failed: {}", e);
                    } else {
                        debug!("Port write succeed:\n{}", packet.show());
                    }
                } else {
                    sleep(Duration::from_millis(10)).await;
                }
            }
        });
    }
}

/// 列出可用序列埠名稱 <br>
/// Tauri command: list available port names
#[tauri::command]
pub async fn cmd_available_port_async() -> Result<Vec<String>, String> {
    let ports = UartAsyncManager::available().await?;
    let names = ports.into_iter().rev().map(|info| info.port_name).collect();
    info!("All available ports: {:?}", names);
    Ok(names)
}

/// 檢查序列埠是否已開啟 <br>
/// Tauri command: check if port is open
#[tauri::command]
pub async fn cmd_check_port_open_async(app: AppHandle) -> bool {
    let global_state = app.state::<GlobalState>();
    let state = global_state.uart_manager.lock().await;
    state.check_open().await.is_ok()
}

/// 開啟指定序列埠 <br>
/// Tauri command: open specified port
#[tauri::command]
pub async fn cmd_open_port_async(app: AppHandle, port_name: String) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let mut state = global_state.uart_manager.lock().await;
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
    let mut port = global_state.uart_manager.lock().await;
    port.close().await.map_err(|e| {
        error!("{}", e);
        e.clone()
    })?;
    let message = "Close port succeed".into();
    info!("{}", message);
    Ok(message)
}
