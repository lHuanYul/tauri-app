use std::{sync::Arc, time::Duration};
use log::{error, info, trace};
use tauri::{AppHandle, Manager};
use serialport::{available_ports, SerialPortInfo};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf}, sync::{watch::{channel, Receiver, Sender}, Mutex}, time::{sleep, timeout}};
use crate::{mods::packet_mod::{UartPacket, PACKET_END_CODE, PACKET_MAX_SIZE}, GlobalState};

use super::log_mod::CODE_TRACE;

/// 非同步讀取單一位元組的超時值（µs）<br>
/// Default timeout for each byte read in µs
const   PORT_READ_TIMEOUT_US:       u64     = 1000;
const   MAX_RECEIVE_BUFFER_SIZE:    usize   = PACKET_MAX_SIZE;

pub struct PortAsyncManager {
    port_name: Option<String>,
    inner: Arc<PortAsyncManagerInner>,
    shutdown_tx: Option<Sender<bool>>,
}
impl PortAsyncManager {
    pub fn new() -> Self {
        Self {
            port_name: None,
            inner: Arc::new(PortAsyncManagerInner::new()),
            shutdown_tx: None,
        }
    }

    /// 取得所有可用序列埠清單<br>
    /// Lists all available serial ports asynchronously
    pub async fn available() -> Result<Vec<SerialPortInfo>, String> {
        let ports = available_ports().map_err(|e| {format!("Get available ports failed: {}", e)})?;
        Ok(ports)
    }

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

    pub async fn check_open(&self) -> Result<(), String> {
        self.inner.check_open().await
    }
}

/// 非同步通訊埠管理器<br>
/// A manager for serial port communication supporting asynchronous operations
///
/// 維護一個可選的 `SerialStream` 實例與當前埠名稱<br>
/// Maintains an optional `SerialStream` instance and the port name
pub struct PortAsyncManagerInner {
    reader: Mutex<Option<ReadHalf<SerialStream>>>,
    writer: Mutex<Option<WriteHalf<SerialStream>>>,
}
impl PortAsyncManagerInner {
    /// 建立全新管理器實例，且尚未打開任何埠<br>
    /// Creates a new `PortAsyncManager` with no open port
    fn new() -> Self {
        Self {
            reader: None.into(),
            writer: None.into(),
        }
    }

    /// 檢查埠是否開啟<br>
    /// Checks whether the serial port is currently open
    async fn check_open(&self) -> Result<(), String> {
        if self.reader.lock().await.is_none() || self.writer.lock().await.is_none() {
            return Err( format!("Port is not open") );
        }
        Ok(())
    }

    /// 非同步讀取一整包資料並解析為 `UartPacket`<br>
    /// Reads exactly 1 + data count + 1 bytes and parses into a `UartPacket`
    ///
    /// # 錯誤 / Errors
    /// * 埠未開啟 / port not open
    /// * 讀取失敗或超時 / read error or timeout
    /// * 解析封包失敗 / packet parse error
    async fn read_packet(&self) -> Result<UartPacket, String> {
        self.check_open().await?;
        let mut reader_guard = self.reader.lock().await;
        let reader = reader_guard.as_mut().unwrap();
        // let start = Instant::now();
        let mut buffer: Vec<u8> = Vec::with_capacity(MAX_RECEIVE_BUFFER_SIZE);
        for _ in 0..MAX_RECEIVE_BUFFER_SIZE {
            match timeout(
                Duration::from_micros(PORT_READ_TIMEOUT_US),
                reader.read_u8(),
            ).await {
                Ok(Ok(byte)) => {
                    buffer.push(byte);
                },
                Ok(Err(e)) => { return Err( format!("Read u8 failed: {}", e) ); }
                Err(_) => {
                    if buffer.is_empty() { return Err( format!("{}Read nothing", CODE_TRACE) ); }
                    if *buffer.last().unwrap() != PACKET_END_CODE {
                        return Err( format!("Read timeout at {} bytes (or no end code)", buffer.len()) );
                    }
                    break;
                }
            };
        }
        // info!("耗時: {:.2?}", start.elapsed());
        let packet = UartPacket::pack(buffer)?;
        Ok(packet)
    }

    /// 非同步將 `UartPacket` 寫入序列埠<br>
    /// Writes the bytes of a `UartPacket` to the serial port asynchronously
    ///
    /// # 錯誤 / Errors
    /// * 埠未開啟 / port not open
    /// * 寫入失敗 / write failure
    async fn write_packet(&self, packet: UartPacket) -> Result<(), String> {
        self.check_open().await?;
        let mut writer_guard = self.writer.lock().await;
        let writer = writer_guard.as_mut().unwrap();
        let buffer = packet.unpack()?;
        writer.write_all(&buffer).await.map_err(|e| format!("Write failed: {}", e))?;
        Ok(())
    }

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
                info!("Port read succeed: {}", packet.show());
                let global = read_handle.state::<GlobalState>();
                let mut receive_buffer = global.receive_buffer.lock().await;
                let _ = receive_buffer.push(packet).map_err(|e| {
                    error!("Packet store failed: {}", e);
                });
            }
        });

        let arc_write = Arc::clone(self);
        let shutdown_write = shutdown_rx.clone();
        let write_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown_write.borrow() { break; }
                let maybe_pkt = {
                    let global = write_handle.state::<GlobalState>();
                    let mut transfer_buffer = global.transfer_buffer.lock().await;
                    let result = transfer_buffer.pop_front();
                    result
                };
                if maybe_pkt.is_none() {
                    sleep(Duration::from_millis(10)).await;
                    continue;
                }
                let _ = arc_write.write_packet(maybe_pkt.unwrap()).await.map_err(|e| {
                    error!("Port write failed: {}", e);
                });
                info!("Port write succeed")
            }
        });
    }
}

#[tauri::command]
/// Tauri command: 取得可用埠名稱清單<br>
/// Tauri command: list available port names
pub async fn cmd_available_port_async() -> Result<Vec<String>, String> {
    let ports = PortAsyncManager::available().await?;
    let names = ports.into_iter().rev().map(|info| info.port_name).collect();
    info!("All available ports: {:?}", names);
    Ok(names)
}

#[tauri::command]
/// Tauri command: 檢查埠是否已開啟<br>
/// Tauri command: check if port is open
pub async fn cmd_check_port_open_async(app: AppHandle) -> bool {
    let global_state = app.state::<GlobalState>();
    let state = global_state.main_port.lock().await;
    state.check_open().await.is_ok()
}

#[tauri::command]
/// Tauri command: 開啟指定埠<br>
/// Tauri command: open specified port
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

#[tauri::command]
/// Tauri command: 關閉目前埠<br>
/// Tauri command: close current port
pub async fn cmd_close_port_async(app: AppHandle) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let mut port = global_state.main_port.lock().await;
    port.close().await.map_err(|e| {
        error!("{}", e);
        e.clone()
    })?;
    let _msg = format!("Open close succeed");
    info!("{}", _msg);
    Ok(_msg)
}

#[tauri::command]
/// Tauri command: 測試封包寫入與讀取<br>
/// Tauri command: test packet write and read
pub async fn cmd_serial_test(app: AppHandle) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let packet = UartPacket::new(vec![1,2,3,4,5,6,7,8,9,10,11,12])?;

    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
    transfer_buffer.push(packet.clone()).map_err(|e| {
        error!("{}", e);
        e.clone()
    })?;
    Ok("Push finish".to_string())
}
