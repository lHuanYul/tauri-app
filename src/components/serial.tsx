import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const SerialComponent = () => {
    const [data1, setData1] = useState(() => localStorage.getItem("data1") || "");
    const [data2, setData2] = useState(() => localStorage.getItem("data2") || "");
    const [data3, setData3] = useState(() => localStorage.getItem("data3") || "");
    const [data4, setData4] = useState(() => localStorage.getItem("data4") || "");
    const [data5, setData5] = useState(() => localStorage.getItem("data5") || "");
    const [data6, setData6] = useState(() => localStorage.getItem("data6") || "");
    const [data7, setData7] = useState(() => localStorage.getItem("data7") || "");
    const [data8, setData8] = useState(() => localStorage.getItem("data8") || "");
    const [response, setResponse] = useState(() => sessionStorage.getItem("TRResponse") || "");

    useEffect(() => { localStorage.setItem("data1", data1); }, [data1]);
    useEffect(() => { localStorage.setItem("data2", data2); }, [data2]);
    useEffect(() => { localStorage.setItem("data3", data3); }, [data3]);
    useEffect(() => { localStorage.setItem("data4", data4); }, [data4]);
    useEffect(() => { localStorage.setItem("data5", data5); }, [data5]);
    useEffect(() => { localStorage.setItem("data6", data6); }, [data6]);
    useEffect(() => { localStorage.setItem("data7", data7); }, [data7]);
    useEffect(() => { localStorage.setItem("data8", data8); }, [data8]);
    useEffect(() => { sessionStorage.setItem("TRResponse", response); }, [response]);

    const serial_c = async () => {
        try {
            const result = await invoke("serial_command", {
                data1: Number(data1),
                data2: Number(data2),
                data3: Number(data3),
                data4: Number(data4),
                data5: Number(data5),
                data6: Number(data6),
                data7: Number(data7),
                data8: Number(data8),
            });
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
            <input
                type="text"
                placeholder="data1"
                value={data1}
                onChange={(e) => setData1(e.target.value)}
                className="border-2 px-4"
            />
            <input
                type="text"
                placeholder="data2"
                value={data2}
                onChange={(e) => setData2(e.target.value)}
                className="border-2 px-4"
            />
            <input
                type="text"
                placeholder="data3"
                value={data3}
                onChange={(e) => setData3(e.target.value)}
                className="border-2 px-4"
            />
            <input
                type="text"
                placeholder="data4"
                value={data4}
                onChange={(e) => setData4(e.target.value)}
                className="border-2 px-4"
            />
            <input
                type="text"
                placeholder="data5"
                value={data5}
                onChange={(e) => setData5(e.target.value)}
                className="border-2 px-4"
            />
            <input
                type="text"
                placeholder="data6"
                value={data6}
                onChange={(e) => setData6(e.target.value)}
                className="border-2 px-4"
            />
            <input
                type="text"
                placeholder="data7"
                value={data7}
                onChange={(e) => setData7(e.target.value)}
                className="border-2 px-4"
            />
            <input
                type="text"
                placeholder="data8"
                value={data8}
                onChange={(e) => setData8(e.target.value)}
                className="border-2 px-4"
            />
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
