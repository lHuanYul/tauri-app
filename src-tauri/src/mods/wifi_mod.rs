use std::{sync::Arc, net::UdpSocket as StdUdpSocket};
use log::{info, warn};
use tokio::{net::{TcpListener, UdpSocket}, sync::{watch::{channel, Receiver, Sender}, Mutex}, io::AsyncReadExt};
use tauri::{AppHandle, Manager};
use crate::{GlobalState, mods::wifi_packet_proc_mod};

const TCP_PORT: &str = "60000";
const UDP_PORT: &str = "60001";

const   MAX_RECEIVE_BUFFER_SIZE: usize = wifi_packet_proc_mod::WIFI_PACKET_MAX_SIZE;

pub struct WifiAsyncManager {
    device_ip: String,
    inner: Arc<WifiAsyncManagerInner>,
    shutdown: Option<Sender<bool>>,
}
impl WifiAsyncManager {
    pub fn new() -> Self {
        let mut new_manager = WifiAsyncManager {
            device_ip: "0.0.0.0".into(),
            inner: Arc::new(WifiAsyncManagerInner::new()),
            shutdown: None,
        };
        if let Ok(sock) = StdUdpSocket::bind("0.0.0.0:0") {
            let _ = sock.connect("8.8.8.8:80");
            if let Ok(addr) = sock.local_addr() {
                new_manager.device_ip = addr.ip().to_string();
            }
        }
        new_manager
    }

    pub async fn start(&mut self, app: AppHandle) -> Result<(), String> {
        // 清除停止旗標
        let (shutdown_tx, shutdown_rx) = channel(false);
        self.shutdown.replace(shutdown_tx);

        let (addr_tcp, addr_udp) = (
            format!("{}:{}", self.device_ip, TCP_PORT),
            format!("{}:{}", self.device_ip, UDP_PORT)
        );
        let tcp_listener = TcpListener::bind(&addr_tcp).await
            .map_err(|e| format!("TCP bind failed: {}", e))?;
        *self.inner.tcp_listener.lock().await = Some(tcp_listener);

        let udp_socket = UdpSocket::bind(&addr_udp).await
            .map_err(|e| format!("UDP bind failed: {}", e))?;
        *self.inner.udp_socket.lock().await = Some(udp_socket);

        // 啟動背景 task
        self.inner.tcp_read_start(app.clone(), shutdown_rx.clone());
        self.inner.udp_read_start(app, shutdown_rx);
        Ok(())
    }

    /// 設定停止旗標並等所有 task 結束
    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(true);
            // 若要 await JoinHandle，可再設計成儲存 handles
        }
        Ok(())
    }
}

struct WifiAsyncManagerInner {
    tcp_listener: Mutex<Option<TcpListener>>,
    udp_socket: Mutex<Option<UdpSocket>>,
}
impl WifiAsyncManagerInner {
    fn new() -> Self {
        Self {
            tcp_listener: Mutex::new(None),
            udp_socket:   Mutex::new(None),
        }
    }

    /// spawn TCP 接收迴圈
    fn tcp_read_start(self: &Arc<Self>, app: AppHandle, shutdown: Receiver<bool>) {
        let arc_handle = Arc::clone(self);
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown.borrow() { break; }
                let mut guard = arc_handle.tcp_listener.lock().await;
                if let Some(listener) = guard.as_mut() {
                    if let Ok((mut stream, peer)) = listener.accept().await {
                        let mut buf = vec![0u8; MAX_RECEIVE_BUFFER_SIZE];
                        if let Ok(n) = stream.read(&mut buf).await {
                            let data = &buf[..n];
                            info!("TCP got {} bytes from {}: {:?}", n, peer, data);
                        }
                    }
                }
            }
        });
    }

    /// spawn UDP 接收迴圈
    fn udp_read_start(self: &Arc<Self>, app: AppHandle, shutdown: Receiver<bool>) {
        let arc_handle = Arc::clone(self);
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown.borrow() { break; }
                let mut guard = arc_handle.udp_socket.lock().await;
                let socket = if let Some(skt) = guard.as_mut() {
                    skt
                } else {
                    continue;
                };
                let mut buf = vec![0u8; MAX_RECEIVE_BUFFER_SIZE];
                let (n, peer) = if let Ok(pair) = socket.recv_from(&mut buf).await {
                    pair
                } else {
                    warn!("UDP recv_from failed");
                    continue;
                };
                let data = &buf[..n];
                info!("UDP got {} bytes from {}: {:?}", n, peer, data);
            }
        });
    }
}

#[tauri::command]
pub async fn cmd_wifi_start(app: AppHandle) -> Result<String, String> {
    let state = app.state::<GlobalState>();
    let mut mgr = state.wifi_manager.lock().await;
    mgr.start(app.clone()).await.map_err(|e| e)?;
    Ok("WiFi listener started".into())
}

#[tauri::command]
pub async fn cmd_wifi_stop(app: AppHandle) -> Result<String, String> {
    let state = app.state::<GlobalState>();
    let mut mgr = state.wifi_manager.lock().await;
    mgr.stop().await.map_err(|e| e)?;
    Ok("WiFi listener stopped".into())
}
