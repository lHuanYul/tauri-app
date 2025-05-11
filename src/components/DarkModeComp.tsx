import { useState, useEffect } from 'react';

interface DarkModeToggleProps {
    className?: string;
}

const DarkModeComp = ({ className }: DarkModeToggleProps) => {
    const [isDark, setIsDark] = useState(() => {
        const saved = localStorage.getItem('darkTheme');
        return saved ? JSON.parse(saved) : false;
    });

    const toggleDarkMode = () => {
        setIsDark((prev: any) => !prev);
    };

    useEffect(() => {
        if (isDark) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
        localStorage.setItem('darkTheme', JSON.stringify(isDark));
    }, [isDark]);

    return (
        <button onClick={toggleDarkMode} className={className ? className : "dark_mode-button-defalt"}>
            {isDark ? "Light" : "Dark"}
        </button>
    );
};

export default DarkModeComp;
