import DarkModeComponent from './DarkModeComponent';

interface PageSelectProps {
    pageName: string;
    setPageName: React.Dispatch<React.SetStateAction<string>>;
}

const PageSelectComponent = ({ pageName, setPageName }: PageSelectProps) => {
    const handleASBClick = async () => {
    };
    const handlePOCClick = () => {
        const name = "open_close_port";
        localStorage.setItem("pageName", name);
        setPageName(name);
    };
    const handlePTrClick = () => {
        const name = "port_transmit";
        localStorage.setItem("pageName", name);
        setPageName(name);
    };
    const handleCGeClick = () => {
        const name = "chart_generate";
        localStorage.setItem("pageName", name);
        setPageName(name);
    };
    const handleMGeClick = () => {
        const name = "map_generate";
        localStorage.setItem("pageName", name);
        setPageName(name);
    };
    const handleTESTClick = async () => {
        const name = "test";
        localStorage.setItem("pageName", name);
        setPageName(name);
        // await invoke("mytest");
    };

    return (
        <div className="w-full text-gray-400 text-center text-xl h-full">
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
                <DarkModeComponent className="py-2 px-4 w-full text-gray-300 hover:bg-gray-700" />
            </div>
            <div className="border-b-2 border-gray-600"></div>
            <button onClick={handlePOCClick} className={pageName === "open_close_port" ? "page_select-button_list-select" : "page_select-button_list-defalt"}>
                Port O/C
            </button>
            <div className="border-b-2 border-gray-600"></div>
            <button onClick={handlePTrClick} className={pageName === "port_transmit" ? "page_select-button_list-select" : "page_select-button_list-defalt"}>
                Port Transmite
            </button>
            <div className="border-b-2 border-gray-600"></div>
            <button onClick={handleCGeClick} className={pageName === "chart_generate" ? "page_select-button_list-select" : "page_select-button_list-defalt"}>
                Chart Gen
            </button>
            <div className="border-b-2 border-gray-600"></div>
            <button onClick={handleMGeClick} className={pageName === "map_generate" ? "page_select-button_list-select" : "page_select-button_list-defalt"}>
                Map Gen
            </button>
            <div className="border-b-2 border-gray-600"></div>
            <button onClick={handleTESTClick} className="page_select-button_list-defalt">
                Test
            </button>
            <div className="border-b-2 border-gray-600"></div>
        </div>
    );
};

export default PageSelectComponent;
