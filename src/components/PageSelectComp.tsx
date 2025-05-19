import DarkModeComp from './DarkModeComp';

// PageSelectProps: 頁面選擇組件的屬性定義
// Props definition for the page selection component
interface PageSelectProps {
    pageName: string;
    // 更新頁面名稱的方法
    // Function to update the page name
    setPageName: React.Dispatch<React.SetStateAction<string>>;
}

// PageSelectComp: 提供頁面切換按鈕與 Dark Mode 切換按鈕的組件
// Component that offers page navigation buttons and a dark mode toggle button
const PageSelectComp = ({ pageName, setPageName }: PageSelectProps) => {
    // handleASBClick: 處理側邊欄按鈕點擊事件（未實作）
    // Handle click for the sidebar button (not implemented)
    const handleASBClick = async () => {
    };

    // handlePOCClick: 選擇 Port O/C 頁面
    // Select the Port Open/Close page
    const handlePOCClick = () => {
        const name = "uart_port_oc";
        localStorage.setItem("pageName", name);
        setPageName(name);
    };

    // handlePTrClick: 選擇 Port Transmit 頁面
    // Select the Port Transmit page
    const handlePTrClick = () => {
        const name = "uart_port_control";
        localStorage.setItem("pageName", name);
        setPageName(name);
    };

    // handleCGeClick: 選擇 Chart Generate 頁面
    // Select the Chart Generate page
    const handleCGeClick = () => {
        const name = "chart_generate";
        localStorage.setItem("pageName", name);
        setPageName(name);
    };

    // handleMGeClick: 選擇 Map Generate 頁面
    // Select the Map Generate page
    const handleMGeClick = () => {
        const name = "map_generate";
        localStorage.setItem("pageName", name);
        setPageName(name);
    };

    // handleTESTClick: 選擇 Test 頁面
    // Select the Test page
    const handleTESTClick = async () => {
        const name = "test";
        localStorage.setItem("pageName", name);
        setPageName(name);
        // await invoke("mytest");
    };

    return (
        <div className="w-full text-gray-400 text-center text-xl h-full">
            {/* 上方工具列: 側邊欄按鈕和 DarkMode 切換按鈕 */}
            {/* Top toolbar: sidebar button and DarkMode toggle button */}
            <div className="flex">
                <button
                    onClick={handleASBClick}
                    className="p-2 w-16 hover:bg-gray-700"
                >
                    <div className="space-y-1.5">
                        <div className="h-0.5 w-7.5 mx-auto bg-gray-300"></div>
                        <div className="h-0.5 w-7.5 mx-auto bg-gray-300"></div>
                        <div className="h-0.5 w-7.5 mx-auto bg-gray-300"></div>
                    </div>
                </button>
                <div className="border-r-2 border-gray-600"></div>
                <DarkModeComp className="py-2 px-4 w-full text-gray-300 hover:bg-gray-700" />
            </div>

            <div className="border-b-2 border-gray-600"></div>

            {/* 各頁面切換按鈕 // Page navigation buttons */}
            <button
                onClick={handlePOCClick}
                className={
                    pageName === "uart_port_oc"
                        ? "page_select-button_list-select"
                        : "page_select-button_list-defalt"
                }
            >
                Port O/C
            </button>
            <div className="border-b-2 border-gray-600"></div>
            <button
                onClick={handlePTrClick}
                className={
                    pageName === "uart_port_control"
                        ? "page_select-button_list-select"
                        : "page_select-button_list-defalt"
                }
            >
                Port Transmit
            </button>
            <div className="border-b-2 border-gray-600"></div>
            <button
                onClick={handleCGeClick}
                className={
                    pageName === "chart_generate"
                        ? "page_select-button_list-select"
                        : "page_select-button_list-defalt"
                }
            >
                Chart Gen
            </button>
            <div className="border-b-2 border-gray-600"></div>
            <button
                onClick={handleMGeClick}
                className={
                    pageName === "map_generate"
                        ? "page_select-button_list-select"
                        : "page_select-button_list-defalt"
                }
            >
                Map Gen
            </button>
            <div className="border-b-2 border-gray-600"></div>
            <button
                onClick={handleTESTClick}
                className="page_select-button_list-defalt"
            >
                Test
            </button>
            <div className="border-b-2 border-gray-600"></div>
        </div>
    );
};

export default PageSelectComp;
