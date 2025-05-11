import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const SerialComponent = () => {
    const [response, setResponse] = useState(() => sessionStorage.getItem("TRResponse") || "");
    useEffect(() => { sessionStorage.setItem("TRResponse", response); }, [response]);

    const serial_c = async () => {
        try {
            const result = await invoke("cmd_serial_test");
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
                onClick={serial_c}
            > Transmit </button>
            <div>
                <pre className="min-h-[8em]">
                    {response || ""}
                </pre>
            </div>
        </div>
    );
};

export default SerialComponent;
