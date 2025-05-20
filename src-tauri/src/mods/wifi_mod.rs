use std::{error, sync::{atomic::{AtomicBool, Ordering}, Arc}, net::UdpSocket as SyncUdpSocket};
use log::{error, info, warn};
use tauri::{App, Manager};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream, UdpSocket}, runtime::Runtime, task::JoinHandle};
use crate::{GlobalState, mods::box_error_mod::box_string_error};

// TCP 發送函式：connect → write_all → close
pub async fn tcp_send_packet() -> Result<(), Box<dyn error::Error>> {
    let addr = "192.168.0.20:60000";
    let data = b"Hello from Rust by TCP";
    // 1. 建立 TCP 連線（三次握手）
    let mut stream = TcpStream::connect(addr).await?;
    info!("TCP connected to {}", addr);

    // 2. 發送整段資料
    stream.write_all(data).await?;
    info!("TCP sent {} bytes to {}", data.len(), addr);

    // 3. 關閉連線
    stream.shutdown().await?;
    Ok(())
}

pub async fn udp_send_packet() -> Result<(), Box<dyn error::Error>> {
    let addr = "192.168.0.20:60001";
    let data = b"Hello from Rust by UDP";

    let socket = UdpSocket::bind("0.0.0.0:0").await
        .map_err(|e| format!("bind failed: {}", e) )?;

    let _ = socket.send_to(data, addr).await;
    Ok(())
}

const TCP_PORT: &str = "60000";
const UDP_PORT: &str = "60001";

pub struct WifiReceive {
    device_ip: String,
    tcp_shutdown: Arc<AtomicBool>,
    tcp_handle: Option<JoinHandle<()>>,
    udp_shutdown: Arc<AtomicBool>,
    udp_handle: Option<JoinHandle<()>>,
}
impl WifiReceive {
    /// 建構新的 WifiTrRe 實例
    pub fn new() -> Self {
        let mut new = WifiReceive {
            device_ip:    "0.0.0.0".into(),
            tcp_shutdown: Arc::new(AtomicBool::new(true)),
            tcp_handle:   None,
            udp_shutdown: Arc::new(AtomicBool::new(true)),
            udp_handle:   None,
        };
        SyncUdpSocket::bind("0.0.0.0:0")
            .map_err(|e| warn!("SyncUdpSocket bind failed: {}", e))
            .map(|socket| {
                socket.connect("8.8.8.8:80")
                    .map_err(|e| warn!("SyncUdpSocket connect failed: {}", e))
                    .ok();
                socket.local_addr()
                    .map_err(|e| warn!("SyncUdpSocket get local_addr failed: {}", e))
                    .map(|addr| new.device_ip = addr.ip().to_string())
                    .ok();
            }).ok();
        new
    }

    /// 啟動 TCP 接收任務
    pub async fn tcp_start(&mut self) -> Result<(), Box<dyn error::Error>> {
        // 清除停止旗標
        self.tcp_shutdown.store(false, Ordering::SeqCst);
        let listener = 
            TcpListener::bind(format!("{}:{}", self.device_ip, TCP_PORT)).await?;
        info!("TCP listening on {}", listener.local_addr()?);

        let shutdown = self.tcp_shutdown.clone();
        let handle = tokio::spawn(async move {
            loop {
                if shutdown.load(Ordering::SeqCst) { break; }
                match listener.accept().await {
                    Ok((mut stream, peer)) => {
                        info!("New TCP connection from {}", peer);
                        let mut buf = [0u8; 1024];
                        loop {
                            if shutdown.load(Ordering::SeqCst) {
                                break;
                            }
                            match stream.read(&mut buf).await {
                                Ok(0) => {
                                    info!("TCP connection {} closed", peer);
                                    break;
                                }
                                Ok(n) => {
                                    info!("TCP read {} bytes from {}: {:?}", n, peer, &buf[..n]);
                                }
                                Err(e) => {
                                    error!("TCP read error from {}: {}", peer, e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("TCP accept error: {}", e);
                    }
                }
            }
        });
        self.tcp_handle = Some(handle);
        Ok(())
    }

    /// 停止 TCP 接收任務
    pub async fn tcp_receive_stop(&mut self) -> Result<(), Box<dyn error::Error>> {
        self.tcp_shutdown.store(true, Ordering::SeqCst);
        match self.tcp_handle.take() {
            None => Err(box_string_error("TCP task is not running")),
            Some(handle) => {
                let _ = handle.await;
                Ok(())
            }
        }
    }

    /// 啟動 UDP 接收迴圈，只要呼叫 stop 就會跳出迴圈
    pub async fn udp_start(&mut self) -> Result<(), Box<dyn error::Error>> {
        // 先重置停止旗標
        self.udp_shutdown.store(false, Ordering::SeqCst);

        // 綁定 socket
        let socket = Arc::new(
            UdpSocket::bind(format!("{}:{}", self.device_ip, UDP_PORT)).await?
        );
        info!("UDP listening on {}", socket.local_addr()?);

        // clone 一份到背景任務
        let shutdown = self.udp_shutdown.clone();
        let socket_clone = socket.clone();
        let handle = tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            loop {
                // 檢查停止旗標
                if shutdown.load(Ordering::SeqCst) { break; }
                // 正常接收
                match socket_clone.recv_from(&mut buf).await {
                    Ok((len, peer)) => {
                        let data = &buf[..len];
                        info!("UDP got {} bytes from {}: {:?}", len, peer, data);
                    }
                    Err(e) => {
                        error!("UDP receive error: {}", e);
                    }
                }
            }
        });
        self.udp_handle = Some(handle);
        Ok(())
    }

    /// 設定停止旗標並等待任務結束
    pub async fn udp_stop(&mut self) -> Result<(), Box<dyn error::Error>> {
        // 觸發停止
        self.udp_shutdown.store(true, Ordering::SeqCst);

        match self.udp_handle.take() {
            // 已經沒有任務在跑，回傳錯誤
            None => Err(box_string_error("UDP task is not running")),
            // 有任務在跑，等待它結束
            Some(handle) => {
                let _ = handle.await;
                Ok(())
            }
        }
    }
}

pub fn setup(app: &mut App) {
    let global_state = app.state::<GlobalState>();
    let rt = Runtime::new().expect("failed to create Tokio runtime");
    rt.block_on(async {
        let mut wifi = global_state.wifi_tr_re.lock().await;

        let _ = wifi.tcp_start().await.map_err(|e| {
            let message = format!("TCP start failed:\n{}", e);
            error!("{}", message)
        });
        let _ = wifi.udp_start().await.map_err(|e| {
            let message = format!("UDP start failed:\n{}", e);
            error!("{}", message)
        });
    });
}
