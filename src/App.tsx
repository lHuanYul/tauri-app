import { useState } from 'react';
import UartPortOCComp from './components/UartPortOCComp';
import UartControlComp from './components/UartControlComp';
import PageSelectComp from './components/PageSelectComp';
import ChartGenerateComp from './components/ChartGenerateComp';
import MapGeneratorComp from './components/MapGeneratorComp';
import MapDisplayComp from './components/MapDisplayComp';
import WifiControlComp from './components/WifiControlComp';

const App = () => {
    /*useEffect(() => {
        localStorage.clear()
    }, [])*/

    const [pageName, setPageName] = useState(
        () => localStorage.getItem("pageName") || "uart_port_oc"
    );

    let content;
    switch (pageName) {
        case "uart_port_oc":
            content = <UartPortOCComp />;
            break;
        case "uart_port_control":
            content = <UartControlComp />;
            break;
        case "wifi_control":
            content = <WifiControlComp />;
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
            localStorage.setItem("pageName", "uart_port_oc");
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
