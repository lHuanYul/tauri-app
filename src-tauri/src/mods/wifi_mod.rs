use std::{error::Error, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket as StdUdpSocket}, sync::Arc, time::Duration};
use log::{debug, error, info, warn};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream, UdpSocket}, sync::{watch::{channel, Receiver, Sender}, Mutex}, time::sleep};
use tauri::{AppHandle, Manager};
use crate::{GlobalState, mods::wifi_packet_mod::{self, WifiPacket}};

const _TARGET_IP: &str = "192.168.0.20";
const TCP_PORT: u16 = 60000;
const UDP_PORT: u16 = 60001;

const   MAX_RECEIVE_BUFFER_SIZE: usize = wifi_packet_mod::WIFI_TCP_PACKET_MAX_SIZE;

pub struct WifiAsyncManager {
    device_ip: IpAddr,
    inner: Arc<WifiAsyncManagerInner>,
    shutdown: Option<Sender<bool>>,
}
impl WifiAsyncManager {
    pub fn new() -> Self {
        let mut device_ip: IpAddr = "0.0.0.0".parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        if let Ok(sock) = StdUdpSocket::bind("0.0.0.0:0") {
            let _ = sock.connect("8.8.8.8:80");
            if let Ok(addr) = sock.local_addr() {
                device_ip = addr.ip();
            }
        }
        Self {
            device_ip,
            inner: Arc::new(WifiAsyncManagerInner::new(device_ip)),
            shutdown: None,
        }
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
        self.inner.udp_read_start(app.clone(), shutdown_rx.clone());
        self.inner.tcp_write_start(app.clone(), shutdown_rx.clone());
        self.inner.udp_write_start(app, shutdown_rx);
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
    device_ip: IpAddr,
    tcp_listener: Mutex<Option<TcpListener>>,
    udp_socket: Mutex<Option<UdpSocket>>,
}
impl WifiAsyncManagerInner {
    fn new(device_ip: IpAddr) -> Self {
        Self {
            device_ip,
            tcp_listener: Mutex::new(None),
            udp_socket:   Mutex::new(None),
        }
    }

    async fn tcp_read_packet(&self) -> Result<WifiPacket, Box<dyn Error + Send + Sync>> {
        let mut guard = self.tcp_listener.lock().await;
        let listener = if let Some(lsn) = guard.as_mut() {
            lsn
        } else {
            return Err("UDP listener not initialized".into());
        };
        let (mut stream, peer) = if let Ok(pair) = listener.accept().await {
            pair
        } else {
            return Err("TCP accept failed".into());
        };
        let mut buf = vec![0u8; MAX_RECEIVE_BUFFER_SIZE];
        let count = if let Ok(n) = stream.read(&mut buf).await {
            n
        } else {
            return Err("TCP read error".into());
        };
        let packet = WifiPacket::new(self.device_ip, &buf[..count])?;
        debug!("TCP got {} bytes from {}: {:?}", count, peer, &buf[..count]);
        Ok(packet)
    }

    /// spawn TCP 接收迴圈
    fn tcp_read_start(self: &Arc<Self>, app: AppHandle, shutdown: Receiver<bool>) {
        let arc_handle = Arc::clone(self);
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown.borrow() { break; }
                match arc_handle.tcp_read_packet().await {
                    Ok(_) => todo!(),
                    Err(_) => todo!(),
                }
            }
        });
    }

    async fn udp_read_packet(self: &Arc<Self>) -> Result<WifiPacket, Box<dyn Error + Send + Sync>> {
        let mut guard = self.udp_socket.lock().await;
        let socket = if let Some(skt) = guard.as_mut() {
            skt
        } else {
            return Err("UDP socket not initialized".into());
        };
        let mut buf = vec![0u8; MAX_RECEIVE_BUFFER_SIZE];
        let (count, peer) = if let Ok(pair) = socket.recv_from(&mut buf).await {
            pair
        } else {
            return Err("UDP recv_from failed".into());
        };
        let packet = WifiPacket::new(self.device_ip, &buf[..count])?;
        debug!("UDP got {} bytes from {}: {:?}", count, peer, &buf[..count]);
        Ok(packet)
    }

    /// spawn UDP 接收迴圈
    fn udp_read_start(self: &Arc<Self>, app: AppHandle, shutdown: Receiver<bool>) {
        let arc_handle = Arc::clone(self);
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown.borrow() { break; }
                match arc_handle.udp_read_packet().await {
                    Ok(_) => todo!(),
                    Err(_) => todo!(),
                } 
            }
        });
    }

    async fn tcp_write_packet(&self, packet: WifiPacket) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (target, data) =
            (SocketAddr::new(packet.target_ip(), TCP_PORT), packet.data());
        let connect = TcpStream::connect(target).await;
        if let Err(e) = connect {
            let message = format!("TCP connect failed: {}", e);
            return Err(message.into());
        }
        let mut stream = connect.unwrap();
        if let Err(e) = stream.write_all(&data).await {
            let message = format!("TCP write failed: {}", e);
            return Err(message.into());
        }
        let message = format!("TCP sent {} bytes to {}", data.len(), target);
        debug!("{}", message);
        Ok(())
    }

    // spawn TCP 寫出迴圈，定時從 AppHandle 取得要發送的資料
    fn tcp_write_start(self: &Arc<Self>, app: AppHandle, shutdown: Receiver<bool>) {
        let arc_handle = Arc::clone(self);
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown.borrow() { break; }
                let global_state = app_handle.state::<GlobalState>();
                let maybe_pkt = {
                    let mut state_buffer = global_state.wifi_tcp_traf_buf.lock().await;
                    state_buffer.pop_front()
                };
                let packet = if let Ok(pkt) = maybe_pkt {
                    pkt
                } else {
                    sleep(Duration::from_millis(10)).await;
                    continue;
                };
                if let Err(e) = arc_handle.tcp_write_packet(packet.clone()).await {
                    error!("TCP write failed: {}", e);
                    continue;
                }
                debug!("TCP write succeed:\n{}", packet.show());
            }
        });
    }

    async fn udp_write_packet(&self, packet: WifiPacket) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let guard = self.udp_socket.lock().await;
        if let Some(socket) = guard.as_ref() {
            let (target, data) = (SocketAddr::new(packet.target_ip(), TCP_PORT), packet.data());
            socket.send_to(&data, target).await
                .map_err(|e| format!("UDP send failed: {}", e).into())
        } else {
            Err("UDP socket not initialized".into())
        }
    }

    fn udp_write_start(self: &Arc<Self>, app: AppHandle, shutdown: Receiver<bool>) {
        let arc = Arc::clone(self);
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                if *shutdown.borrow() { break; }
                let global_state = app_handle.state::<GlobalState>();
                let maybe_pkt = {
                    let mut state_buffer = global_state.wifi_udp_traf_buf.lock().await;
                    state_buffer.pop_front()
                };
                let packet = if let Ok(pkt) = maybe_pkt {
                    pkt
                } else {
                    sleep(Duration::from_millis(10)).await;
                    continue;
                };
                if let Err(e) = arc.udp_write_packet(packet.clone()).await {
                    warn!("UDP send failed: {}", e);
                }
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
