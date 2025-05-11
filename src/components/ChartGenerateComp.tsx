import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

const ChartGenerateComp: React.FC = () => {
    const [imgSrc, setImgSrc] = useState<string>('');

    useEffect(() => {
        const interval = setInterval(async () => {
            try {
                // 每秒呼叫 update_chart，若累計到 10 筆資料則回傳圖像 Base64
                const response: string | null = await invoke('chart_generate');
                if (response) {
                    setImgSrc(`data:image/png;base64,${response}`);
                }
            } catch (error) {
                console.error('更新圖像失敗:', error);
            }
        }, 200); // 每秒更新一次

        return () => clearInterval(interval);
    }, []);

    return (
        <div>
            <h2>後端生成圖像</h2>
            {imgSrc ? (
                <img src={imgSrc} alt="Chart generated from backend" />
            ) : (
                <p>正在累計資料，尚未生成圖像...</p>
            )}
        </div>
    );
};

export default ChartGenerateComp;
