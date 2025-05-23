import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const UartControlComp = () => {
    const [response, setResponse] = useState(() => sessionStorage.getItem("TRResponse") || "");
    useEffect(() => { sessionStorage.setItem("TRResponse", response); }, [response]);

    const cmd_stop = async () => {
        try {
            const result = await invoke("cmd_send_spd_stop");
            setResponse(result as string);
        } catch (error: any) {
            setResponse(`錯誤：${error}`);
        }
    };
    const cmd_once = async () => {
        try {
            const result = await invoke("cmd_send_spd_once");
            setResponse(result as string);
        } catch (error: any) {
            setResponse(`錯誤：${error}`);
        }
    };
    const cmd_start = async () => {
        try {
            const result = await invoke("cmd_send_spd_start");
            setResponse(result as string);
        } catch (error: any) {
            setResponse(`錯誤：${error}`);
        }
    };

    return (
        <div className="
            flex flex-col p-4 space-y-4
            dark:bg-gray-700 text-white text-xl md:text-3xl"
        >
            <button
                className="bg-blue-500 hover:bg-blue-600 text-white py-1 px-4 rounded"
                onClick={cmd_stop}
            > cmd_stop </button>
            <button
                className="bg-blue-500 hover:bg-blue-600 text-white py-1 px-4 rounded"
                onClick={cmd_once}
            > cmd_once </button>
            <button
                className="bg-blue-500 hover:bg-blue-600 text-white py-1 px-4 rounded"
                onClick={cmd_start}
            > cmd_start </button>
            <div>
                <pre className="min-h-[8em]">
                    {response || ""}
                </pre>
            </div>
        </div>
    );
};

export default UartControlComp;
