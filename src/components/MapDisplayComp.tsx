import React, { useMemo } from 'react';
import { Graph } from 'react-d3-graph';
import mapInfo from '../../src-tauri/generate/map/map_info.json';

// 方向對應角度（度）：index 0=北，1=東北，2=東，…
const dirAngles = [-90, -45, 0, 45, 90, 135, 180, -135];

interface MapNode { id: number; name: string; connect: { pos: number; len: number }[]; }
type NodeType = { id: string; label: string; x: number; y: number };
type LinkType = { source: string; target: string; label: string };

const MapDisplayComp: React.FC = () => {
    const width = 800;
    const height = 600;
    const centerX = width / 2;
    const centerY = height / 2;
    const scale = 20; // 每單位長度像素比例

    const data = useMemo(() => {
        const home = mapInfo.find(n => n.id === 1)!;

        const nodes: NodeType[] = mapInfo.map(node => {
            if (node.id === 1) {
                return { id: '1', label: node.name, x: centerX, y: centerY };
            }
            // 找 home.connect 中指向此 node 的索引和資料
            const idx = home.connect.findIndex(c => c.pos === node.id && c.len > 0);
            if (idx >= 0) {
                const len = home.connect[idx].len;
                const angle = (dirAngles[idx] * Math.PI) / 180;
                return {
                    id: node.id.toString(),
                    label: node.name,
                    x: centerX + Math.cos(angle) * len * scale,
                    y: centerY + Math.sin(angle) * len * scale,
                };
            }
            // 若無從 home 連線，暫放中心
            return { id: node.id.toString(), label: node.name, x: centerX, y: centerY };
        });

        const links: LinkType[] = mapInfo.flatMap(node =>
            node.connect
                .filter(c => c.len > 0)
                .map(c => ({
                    source: node.id.toString(),
                    target: c.pos.toString(),
                    label: `${c.len}`,
                }))
        );

        return { nodes, links };
    }, []);

    const config = useMemo(() => ({
        directed: true,
        staticGraph: true,
        nodeHighlightBehavior: true,
        node: { color: '#a0aec0', size: 400, highlightStrokeColor: '#2b6cb0', fontSize: 14, fontColor: '#2d3748' },
        link: { arrows: { to: true, from: true }, renderLabel: true, fontSize: 12, pointerLength: 5 },
        d3: { disableLinkForce: true },
    }), []);

    return (
        <div className="p-4 bg-white rounded-2xl shadow h-full w-full">
            <h2 className="text-xl font-semibold mb-4">路線圖</h2>
            <Graph id="route-graph" data={data} config={config} />
        </div>
    );
};

export default MapDisplayComp;
