import { useState, useEffect } from 'react';

// DarkModeToggleProps: 按鈕可接收的屬性
// Props for the toggle button component
interface DarkModeToggleProps {
    className?: string;
}

// DarkModeComp: 切換深色模式的元件
// Component to toggle dark mode
const DarkModeComp = ({ className }: DarkModeToggleProps) => {
    // isDark: 狀態表示是否為深色模式
    // State indicating if dark mode is enabled
    const [isDark, setIsDark] = useState(() => {
        const saved = localStorage.getItem('darkTheme');
        return saved ? JSON.parse(saved) : false;
    });

    // 切換深色模式的函式
    // Function to toggle dark mode
    const toggleDarkMode = () => {
        setIsDark((prev: boolean) => !prev);
    };

    // 當 isDark 改變時，更新 HTML 類別與 localStorage 
    // Update document class and localStorage on isDark change
    useEffect(() => {
        if (isDark) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
        localStorage.setItem('darkTheme', JSON.stringify(isDark));
    }, [isDark]);

    return (
        <button
            onClick={toggleDarkMode}
            className={
                className
                    ? className
                    : "dark_mode-button-default"
            }
        >
            {isDark ? "Light" : "Dark"}
        </button>
    );
};

export default DarkModeComp;
