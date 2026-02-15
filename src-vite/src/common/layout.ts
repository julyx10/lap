export interface Geometry {
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface JustifiedLayoutResult {
    boxes: Geometry[];
    containerHeight: number;
}

export function calculateJustifiedLayout(
    items: any[],
    containerWidth: number,
    targetHeight: number,
    spacing: number | { x: number, y: number }
): JustifiedLayoutResult {
    const spacingX = typeof spacing === 'number' ? spacing : spacing.x;
    const spacingY = typeof spacing === 'number' ? spacing : spacing.y;

    const boxes: Geometry[] = [];
    let currentRow: any[] = [];
    let currentY = 0;

    // Helper to process a completed row
    const processRow = (rowItems: any[], isLastRow: boolean) => {
        if (rowItems.length === 0) return;

        // Calculate total aspect ratio of the row
        let totalAspectRatio = 0;
        rowItems.forEach(item => {
            let w = item.width || 100;
            let h = item.height || 100;
            if (item.rotate && item.rotate % 180 !== 0) {
                [w, h] = [h, w];
            }
            totalAspectRatio += w / h;
        });

        // Calculate available width for content (minus spacing)
        const availableWidth = containerWidth - (rowItems.length - 1) * spacingX;

        let rowHeight = targetHeight;

        if (!isLastRow) {
            // For full rows, calculate exact height to fill container
            rowHeight = availableWidth / totalAspectRatio;
        } else {
            // For the last row, usually don't scale up aggressively
            // But if it's very close to full, maybe justify it? 
            // Standard behavior: keep targetHeight, right side can be empty
        }

        // Generate geometry for each item in this row
        let currentX = 0;
        rowItems.forEach((item, index) => {
            let w = item.width || 100;
            let h = item.height || 100;
            if (item.rotate && item.rotate % 180 !== 0) {
                [w, h] = [h, w];
            }
            const aspectRatio = w / h;

            const itemWidth = rowHeight * aspectRatio;

            boxes.push({
                x: currentX,
                y: currentY,
                width: itemWidth,
                height: rowHeight
            });

            currentX += itemWidth + spacingX;
        });

        currentY += rowHeight + spacingY;
    };

    let currentRowWidth = 0;

    items.forEach(item => {
        let w = item.width || 100;
        let h = item.height || 100;
        if (item.rotate && item.rotate % 180 !== 0) {
            [w, h] = [h, w];
        }
        // Width if scaled to targetHeight
        const scaledWidth = (w / h) * targetHeight;

        // Check if adding this item exceeds container (roughly)
        // We add spacing only if it's not the first item
        const spacingCost = currentRow.length > 0 ? spacingX : 0;

        if (currentRowWidth + spacingCost + scaledWidth > containerWidth) {
            // Row is full. 
            // We need to decide whether to include this item or wrap before it.
            // Simple greedy approach: if it overflows, wrap now.
            // Justified Layout libraries sometimes check which minimizes error.
            // For simplicity, let's wrap *before* this item if adding it makes it overflow significantly.
            // Actually, standard approach: add items until > containerWidth, then squeeze.

            // Let's try adding it first, see if the row height drops too much?
            // No, simple approach: check if current width is already reasonably full?
            // Or just: accumulate width. If > containerWidth, that's a break point.

            // Let's stick to the threshold logic:
            // If we are already > containerWidth, we MUST break.
            // But usually we break *closest* to containerWidth.

            // Let's use the simple standard algorithm: overflow triggers new row.
            // Wait, if I wrap *before* this item, the previous row might be too short (and thus very tall).
            // If I wrap *after* this item, the row is too long (and thus very short).
            // We want rowHeight to stand close to targetHeight.

            // Let's calculate cost of breaking before vs after.
            // Cost = abs(resultingRowHeight - targetHeight)

            // 1. Break BEFORE adding this item:
            // previous row has `currentRow`.
            const prevAspect = currentRow.reduce((acc, it) => {
                let w = it.width || 100;
                let h = it.height || 100;
                if (it.rotate && it.rotate % 180 !== 0) {
                    [w, h] = [h, w];
                }
                return acc + w / h;
            }, 0);
            const prevWidthAvail = containerWidth - Math.max(0, currentRow.length - 1) * spacingX;
            const heightIfBreakBefore = prevWidthAvail / prevAspect;

            // 2. Break AFTER adding this item (include it):
            const newRow = [...currentRow, item];
            const newAspect = prevAspect + (w / h);
            const newWidthAvail = containerWidth - Math.max(0, newRow.length - 1) * spacingX;
            const heightIfBreakAfter = newWidthAvail / newAspect;

            const diffBefore = Math.abs(heightIfBreakBefore - targetHeight);
            const diffAfter = Math.abs(heightIfBreakAfter - targetHeight);

            if (diffBefore < diffAfter) {
                // Break before is better (closer to target height)
                processRow(currentRow, false);
                currentRow = [item];
                currentRowWidth = scaledWidth;
            } else {
                // Include this item, then break
                processRow(newRow, false);
                currentRow = [];
                currentRowWidth = 0;
                // Note: we just processed the item as part of the previous row. 
                // So we don't add it to next row.
                // Wait, logic implementation detail:
                // if we included it, we are done with it.
                // if we didn't, it becomes the start of next row.
            }
        } else {
            currentRow.push(item);
            currentRowWidth += scaledWidth + spacingCost;
        }
    });

    // Process remaining items as last row
    if (currentRow.length > 0) {
        processRow(currentRow, true);
    }

    // Remove the last spacing from containerHeight
    const finalHeight = currentY > 0 ? currentY - spacingY : 0;

    return {
        boxes,
        containerHeight: finalHeight
    };
}

export interface LinearRowLayoutResult {
    boxes: Geometry[];
    containerWidth: number;
}

export function calculateLinearRowLayout(
    items: any[],
    targetHeight: number,
    spacing: number
): LinearRowLayoutResult {
    const boxes: Geometry[] = [];
    let currentX = 0;

    items.forEach(item => {
        let w = item.width && item.width > 0 ? item.width : 100;
        let h = item.height && item.height > 0 ? item.height : 100;
        if (item.rotate && item.rotate % 180 !== 0) {
            [w, h] = [h, w];
        }
        const aspectRatio = w / h;

        const itemWidth = targetHeight * aspectRatio;

        boxes.push({
            x: currentX,
            y: 0,
            width: itemWidth,
            height: targetHeight
        });

        currentX += itemWidth + spacing;
    });

    // Remove trailing spacing from total width if items exist
    if (boxes.length > 0) {
        currentX -= spacing;
    }

    return {
        boxes,
        containerWidth: currentX
    };
}
