// PortOCComponent：用於開啟或關閉通訊埠的元件 / Component for opening and closing serial ports
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const PortOCComponent = () => {
    // ports：可用埠清單
    // list of available ports
    const [ports, setPorts] = useState<string[]>([]);
    // selectedPort：目前選中的埠 / currently selected port
    const [selectedPort, setSelectPort] = useState(
        () => localStorage.getItem("selectedPort") || ""
    );
    // isOpen：埠是否已開啟
    // whether the port is open
    const [isOpen, setIsOpen] = useState(false);
    // response：後端命令回傳訊息 / backend command response message
    const [response, setResponse] = useState(
        () => sessionStorage.getItem("OCResponse") || ""
    );
    
    // 同步 selectedPort 到 localStorage
    // sync selectedPort to localStorage
    useEffect(() => {
        localStorage.setItem("selectedPort", selectedPort);
    }, [selectedPort]);

    // 同步 response 到 sessionStorage
    // sync response to sessionStorage
    useEffect(() => {
        sessionStorage.setItem("OCResponse", response);
    }, [response]);

    // 初始化時載入可用埠
    // fetch available ports on mount
    useEffect(() => {
        async function fetchPorts() {
            const list = await invoke<string[]>("cmd_available_port_async");
            setPorts(list);
            // 若無選擇或選擇已失效，清空選項
            // clear selection if invalid
            if (!selectedPort || !list.includes(selectedPort)) {
                setSelectPort("");
            }
        }
        fetchPorts();
    }, []);

    // response 變化時檢查埠開啟狀態
    // check port status when response changes
    useEffect(() => {
        async function checkPort() {
            const result = await invoke<boolean>("cmd_check_port_open_async");
            setIsOpen(result);
        }
        checkPort();
    }, [response]);

    // openPort：呼叫後端開埠
    // call backend to open port
    const openPort = async () => {
        const result = await invoke("cmd_open_port_async", { portName: selectedPort });
        const message = `${result}`;
        setResponse(message);
        setIsOpen(true);
    };

    // closePort：呼叫後端關埠
    // call backend to close port
    const closePort = async () => {
        const result = await invoke("cmd_close_port_async");
        const message = `${result}`;
        setResponse(message);
        setIsOpen(false);
    };

    // 元件呈現
    // component render
    return (
        <div className="flex flex-col items-center space-y-4 py-4 text-xl">
            {/* 下拉選單：選擇埠 / dropdown for selecting port */}
            <select
                value={selectedPort}
                onChange={(e) => setSelectPort(e.target.value)}
                className="open_close_port-input-defalt"
            >
                {ports.map((p) => (
                    <option key={p} value={p} className="open_close_port-input-defalt">
                        {p}
                    </option>
                ))}
            </select>
            {/* 根據 isOpen 顯示 Open/Close 按鈕 / toggle Open/Close button based on isOpen */}
            {isOpen ? (
                <button onClick={closePort} className="open_close_port-button-defalt">
                    Close
                </button>
            ) : (
                <button onClick={openPort} className="open_close_port-button-defalt">
                    Open
                </button>
            )}
            {/* 顯示指令回應訊息 / display command response */}
            <div>
                <pre className="text-2xl min-h-[4em]">
                    {response || ""}
                </pre>
            </div>
        </div>
    );
};

export default PortOCComponent;
