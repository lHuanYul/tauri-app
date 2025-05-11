// 地圖生成元件 / Map Generation Component
import { invoke } from '@tauri-apps/api/core';
import React, { useEffect, useState } from 'react';

// 定義 MappingItem 介面，包含 id、名稱與連接資訊 / Define MappingItem interface with id, name, and connection info
interface MappingItem {
    id: number; // 唯一識別碼 / Unique identifier
    name: string; // 元件名稱 / Component name
    connect: { pos: number; len: number }[]; // pos/len 陣列 / Array of pos/len pairs
}

const MapGeneratorComponent: React.FC = () => {
    // 常數：位置與長度上下限
    // Constants: max values for pos and len
    const U16_MAX = 65535; // 16 位元上限 / 16-bit max
    const U32_MAX = 4294967295; // 32 位元上限 / 32-bit max
    const POS_MAX = U16_MAX;
    const LEN_MAX = U32_MAX;

    // 初始 MappingItem 範例
    // Example initial MappingItem
    const initialItem: MappingItem = {
        id: 1,
        name: '',
        connect: Array(8).fill(0).map(() => ({ pos: 0, len: 0 })),
    };

    // state: 所有映射項目
    // state: all mapping items
    // 從 sessionStorage 讀取並解析，無資料則使用 initialItem
    // load from sessionStorage or fallback to initialItem
    const [items, setItems] = useState<MappingItem[]>(() => {
        const stored = sessionStorage.getItem('mapItems');
        if (stored) {
            try {
                return JSON.parse(stored) as MappingItem[];
            } catch {
                return [initialItem];
            }
        }
        return [initialItem];
    });

    // state: 下一個可用的 id
    // state: next available id
    // 從 sessionStorage 讀取，若無則計算出最大 id + 1
    // load or compute max id + 1
    const [nextId, setNextId] = useState<number>(() => {
        const s = sessionStorage.getItem('mapNextId');
        const n = s ? parseInt(s, 10) : NaN;
        if (!isNaN(n)) {
            return n;
        }
        return items.length > 0
            ? Math.max(...items.map(i => i.id)) + 1
            : 2;
    });

    // 將 items 與 nextId 同步到 sessionStorage
    // sync items and nextId to sessionStorage
    useEffect(() => {
        sessionStorage.setItem('mapItems', JSON.stringify(items));
        sessionStorage.setItem('mapNextId', nextId.toString());
    }, [items, nextId]);

    // 後端命令：載入已保存的地圖 JSON
    // backend command: load saved map JSON
    const mapLoad = async () => {
        const json = await invoke<string>('map_load');
        const parsed = JSON.parse(json) as MappingItem[];
        if (parsed.length > 0) {
            setItems(parsed); // 更新 items / update items
            const maxId = Math.max(...parsed.map(i => i.id));
            setNextId(maxId + 1); // 設定下一個 id / set nextId
        } else {
            setItems([initialItem]);
            setNextId(2);
        }
    };

    // 後端命令：保存目前設定
    // backend command: save current settings
    const mapSave = async () => {
        await invoke<string>('map_save', { data: JSON.stringify(items) });
    };

    // 新增一筆映射項目 / add a mapping item
    const addItem = () => {
        if (nextId > POS_MAX) {
            return; // 超過上限則跳過 / skip if exceeding max
        }
        setItems([
            ...items,
            {
                id: nextId,
                name: '',
                connect: Array(8).fill(0).map(() => ({ pos: 0, len: 0 })),
            },
        ]);
        setNextId(nextId + 1);
    };

    // 刪除指定 id 的映射項目，並重新編號
    // remove an item by id and reindex
    const removeItem = (id: number) => {
        const filtered = items
            .filter(item => item.id !== id)
            .map((item, idx) => ({ ...item, id: idx + 1 }));
        setItems(filtered);
        setNextId(filtered.length + 1);
    };

    const direction = (id: number) => {
        switch (id) {
            case 0: return "North";
            case 1: return "EN";
            case 2: return "East";
            case 3: return "ES";
            case 4: return "South";
            case 5: return "WS";
            case 6: return "West";
            case 7: return "WN";
            default: return "X";
        }
    }

    // 更新指定項目的名稱
    // update name of an item
    const updateItemName = (id: number, name: string) => {
        setItems(
            items.map(item => (item.id === id ? { ...item, name } : item))
        );
    };

    // 更新 pos 或 len 的值
    // update pos or len for a connection
    const updateItemConnect = (
        id: number,
        idx: number,
        key: 'pos' | 'len',
        raw: number
    ) => {
        const val = Number.isFinite(raw) ? raw : 0; // 無效輸入歸零 / invalid input to zero
        setItems(
            items.map(item => {
                if (item.id !== id) {
                    return item;
                }
                const conns = item.connect.map((c, i) =>
                    i === idx ? { ...c, [key]: val } : c
                );
                return { ...item, connect: conns };
            })
        );
    };

    // 元件輸出
    // component render
    return (
        <div className="flex flex-col gap-4">
            {/* 保存與載入按鈕 / Save and Load buttons */}
            <div className="flex gap-2">
                <button
                    onClick={mapSave}
                    className="map_generate-button-defalt w-40"
                >
                    Map Save
                </button>
                <button
                    onClick={mapLoad}
                    className="map_generate-button-defalt w-40"
                >
                    Map Load
                </button>
            </div>

            {/* 動態生成的映射項目列表 / dynamic list of mapping items */}
            {items.map(item => (
                <div key={item.id} className="border p-4">
                    {/* 頁首：新增、刪除、名稱輸入與顯示 ID / header: add, remove, name input and show ID */}
                    <div className="flex items-center gap-2 mb-4">
                        <button
                            onClick={addItem}
                            className="map_generate-button-adrm"
                        >
                            +
                        </button>
                        <button
                            onClick={() => removeItem(item.id)}
                            className="map_generate-button-adrm"
                        >
                            -
                        </button>
                        <span className="text-2xl font-bold"> {item.id} </span>
                        <input
                            type="text"
                            placeholder="Component Name"
                            value={item.name}
                            onChange={e => updateItemName(item.id, e.target.value)}
                            className="map_generate-input-defalt flex-1"
                        />
                    </div>
                    {/* pos/len 輸入欄位，分兩排顯示，每排 4 欄 / pos/len inputs in 2 rows of 4 columns */}
                    <div className="grid grid-cols-4 gap-4">
                        {item.connect.map((connect, idx) => (
                            <div key={idx} className="flex flex-col">
                                <span className="text-xl"> {direction(idx)} </span>
                                <input
                                    type="number" min={0} max={POS_MAX} step={1}
                                    placeholder={`CON`}
                                    value={connect.pos === 0 ? "" : connect.pos}
                                    onChange={e =>
                                        updateItemConnect(
                                            item.id,
                                            idx,
                                            'pos',
                                            e.target.valueAsNumber
                                        )
                                    }
                                    className="map_generate-input-defalt mb-1.5"
                                />
                                <input
                                    type="number" min={0} max={LEN_MAX} step={1}
                                    placeholder={`LEN`}
                                    value={connect.len === 0 ? "" : connect.len}
                                    onChange={e =>
                                        updateItemConnect(
                                            item.id,
                                            idx,
                                            'len',
                                            e.target.valueAsNumber
                                        )
                                    }
                                    className="map_generate-input-defalt"
                                />
                            </div>
                        ))}
                    </div>
                </div>
            ))}
        </div>
    );
};

export default MapGeneratorComponent;
