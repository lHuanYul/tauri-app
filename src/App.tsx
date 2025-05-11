import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import PortOCComponent from './components/PortOCComponent';
import SerialComponent from './components/SerialComponent';
import PageSelectComponent from './components/PageSelectComponent';
import ChartGenerateComponent from './components/ChartGenerateComponent';
import MapGeneratorComponent from './components/MapGeneratorComponent';
import MapDisplayComponent from './components/MapDisplayComponent';

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
            content = <PortOCComponent />;
            break;
        case "port_transmit":
            content = <SerialComponent />;
            break;
        case "chart_generate":
            content = <ChartGenerateComponent />;
            break;
        case "map_generate":
            content = <MapGeneratorComponent />;
            break;
        case "test":
            content = <><MapDisplayComponent /></>;
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
                <PageSelectComponent pageName={pageName} setPageName={setPageName} />
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
