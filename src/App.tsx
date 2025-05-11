import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import PortOCComp from './components/PortOCComp';
import SerialComp from './components/SerialComp';
import PageSelectComp from './components/PageSelectComp';
import ChartGenerateComp from './components/ChartGenerateComp';
import MapGeneratorComp from './components/MapGeneratorComp';
import MapDisplayComp from './components/MapDisplayComp';

const App = () => {
    /*useEffect(() => {
        localStorage.clear()
    }, [])*/

    const [pageName, setPageName] = useState(
        () => localStorage.getItem("pageName") || "open_close_port"
    );

    useEffect(() => {
        const interval = setInterval(async () => { await invoke('cmd_1kms_loop'); }, 1000); // ms
        return () => clearInterval(interval);
    }, []);
    useEffect(() => {
        const interval = setInterval(async () => { await invoke('cmd_50ms_loop'); }, 50);
        return () => clearInterval(interval);
    }, []);

    let content;
    switch (pageName) {
        case "open_close_port":
            content = <PortOCComp />;
            break;
        case "port_transmit":
            content = <SerialComp />;
            break;
        case "chart_generate":
            content = <ChartGenerateComp />;
            break;
        case "map_generate":
            content = <MapGeneratorComp />;
            break;
        case "test":
            content = <><MapDisplayComp /></>;
            break;
        default:
            content = <div>Page Not Found</div>;
            break;
    }

    return (
        <div className="flex">
            <aside className="
                text-gray-400 text-center text-xl w-64 h-screen border-r-2 bg-gray-800 border-gray-600"
            >
                <PageSelectComp pageName={pageName} setPageName={setPageName} />
            </aside>
            <main className="
                text-black bg-gray-300 w-full md:text-3xl p-4 space-y-4
                dark:text-gray-300 dark:bg-gray-800"
            >
                {content}
            </main>
        </div>
    );
};

export default App;
