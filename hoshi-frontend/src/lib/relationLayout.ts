import type { RelationEdge, RelationGraph, RelationNode } from "@/api/content/types";

export interface PositionedNode extends RelationNode {
    x: number;
    y: number;
}

export interface LayoutResult {
    nodes: PositionedNode[];
    edges: RelationEdge[];
    width: number;
    height: number;
}

export const NODE_W = 150;
export const NODE_H = 220;
const COLUMN_GAP = 120;
const ROW_GAP = 80;

const UPWARD_RELATIONS = new Set(['source', 'parent', 'adaptation']);
const HORIZONTAL_RELATIONS = new Set(['sequel', 'prequel']);

export function computeLayout(graph: RelationGraph, rootCid: string): LayoutResult {
    const adjacency = new Map<string, RelationEdge[]>();
    graph.edges.forEach((e) => {
        if (!adjacency.has(e.sourceCid)) adjacency.set(e.sourceCid, []);
        if (!adjacency.has(e.targetCid)) adjacency.set(e.targetCid, []);
        adjacency.get(e.sourceCid)!.push(e);
        adjacency.get(e.targetCid)!.push(e);
    });

    const grid = new Map<string, { x: number, y: number }>();
    const occupied = new Set<string>();

    grid.set(rootCid, { x: 0, y: 0 });
    occupied.add(`0,0`);

    // PASS 1: horizontal timeline (sequel/prequel)
    const timelineQueue = [rootCid];
    const timelineVisited = new Set<string>([rootCid]);

    while (timelineQueue.length) {
        const cur = timelineQueue.shift()!;
        const curPos = grid.get(cur)!;
        const edges = adjacency.get(cur) ?? [];

        for (const edge of edges) {
            if (!HORIZONTAL_RELATIONS.has(edge.relationType)) continue;

            const isSource = edge.sourceCid === cur;
            const other = isSource ? edge.targetCid : edge.sourceCid;
            if (grid.has(other) || timelineVisited.has(other)) continue;
            timelineVisited.add(other);

            const fx = edge.relationType === 'sequel'
                ? (isSource ? 1 : -1)
                : (isSource ? -1 : 1);

            let nx = curPos.x + fx;
            let ny = curPos.y;
            while (occupied.has(`${nx},${ny}`)) nx += fx > 0 ? 1 : -1;

            grid.set(other, { x: nx, y: ny });
            occupied.add(`${nx},${ny}`);
            timelineQueue.push(other);
        }
    }

    // PASS 2: vertical relations — fan children out horizontally around parent
    // Process level by level so parents are placed before children
    const verticalQueue: string[] = [...grid.keys()];
    const verticalVisited = new Set<string>(grid.keys());

    while (verticalQueue.length) {
        const cur = verticalQueue.shift()!;
        const curPos = grid.get(cur)!;
        const edges = adjacency.get(cur) ?? [];

        // Collect unplaced children going down and up separately
        const downChildren: string[] = [];
        const upChildren: string[] = [];

        for (const edge of edges) {
            if (HORIZONTAL_RELATIONS.has(edge.relationType)) continue;
            const isSource = edge.sourceCid === cur;
            const other = isSource ? edge.targetCid : edge.sourceCid;
            if (grid.has(other) || verticalVisited.has(other)) continue;

            verticalVisited.add(other);

            if (UPWARD_RELATIONS.has(edge.relationType)) {
                if (isSource) upChildren.push(other);
                else downChildren.push(other);
            } else {
                if (isSource) downChildren.push(other);
                else upChildren.push(other);
            }
        }

        // Fan out downward children: spread horizontally centered on parent
        placeChildren(downChildren, curPos, 1, grid, occupied);
        // Fan out upward children: spread horizontally centered on parent
        placeChildren(upChildren, curPos, -1, grid, occupied);

        verticalQueue.push(...downChildren, ...upChildren);
    }

    // Convert grid coords to pixel positions
    const positionedMap = new Map<string, PositionedNode>();
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

    for (const node of graph.nodes) {
        const pos = grid.get(node.cid);
        if (pos) {
            minX = Math.min(minX, pos.x);
            minY = Math.min(minY, pos.y);
            maxX = Math.max(maxX, pos.x);
            maxY = Math.max(maxY, pos.y);
        }
    }

    const colWidth = NODE_W + COLUMN_GAP;
    const rowHeight = NODE_H + ROW_GAP;

    for (const node of graph.nodes) {
        const pos = grid.get(node.cid);
        if (pos) {
            positionedMap.set(node.cid, {
                ...node,
                x: (pos.x - minX) * colWidth,
                y: (pos.y - minY) * rowHeight
            });
        }
    }

    return {
        nodes: Array.from(positionedMap.values()),
        edges: graph.edges,
        width: (maxX - minX + 1) * colWidth + NODE_W,
        height: (maxY - minY + 1) * rowHeight + NODE_H
    };
}

function placeChildren(
    children: string[],
    parentPos: { x: number; y: number },
    fy: 1 | -1,
    grid: Map<string, { x: number; y: number }>,
    occupied: Set<string>
) {
    if (children.length === 0) return;

    const targetY = parentPos.y + fy;

    const half = Math.floor(children.length / 2);
    const startOffset = -half;

    for (let i = 0; i < children.length; i++) {
        const cid = children[i];
        let nx = parentPos.x + startOffset + i;
        let ny = targetY;

        let rowShift = 0;
        while (occupied.has(`${nx},${ny}`)) {
            rowShift++;
            ny = targetY + fy * rowShift;
        }

        grid.set(cid, { x: nx, y: ny });
        occupied.add(`${nx},${ny}`);
    }
}

export const RELATION_COLORS: Record<string, string> = {
    sequel: '#3b82f6',
    prequel: '#3b82f6',
    parent: '#a855f7',
    side_story: '#a855f7',
    adaptation: '#22c55e',
    summary: '#f59e0b',
    compilation: '#f59e0b',
    contains: '#f59e0b',
    source: '#f59e0b',
    alternative: '#6b7280'
};

export function relationColor(type: string): string {
    return RELATION_COLORS[type] ?? '#6b7280';
}

export function relationLabel(type: string): string {
    return type.replace(/_/g, ' ').replace(/^\w/, c => c.toUpperCase());
}