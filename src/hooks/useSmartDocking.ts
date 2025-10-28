import { useState, useEffect, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface Position {
  x: number;
  y: number;
}

interface SmartDockingOptions {
  dockWidth: number;
  dockHeight: number;
  margin: number;
  snapToCorner?: boolean;
}

export function useSmartDocking({
  dockWidth = 420,
  dockHeight = 640,
  margin = 24,
  snapToCorner = false,
}: SmartDockingOptions) {
  const [position, setPosition] = useState<Position>({ x: 0, y: 0 });
  const [isDocked, setIsDocked] = useState(false);

  const calculateOptimalPosition = useCallback(
    async (cursorX?: number, cursorY?: number) => {
      try {
        const currentWindow = getCurrentWindow();
        // @ts-ignore - Tauri Window API
        const monitor = await currentWindow.currentMonitor();

        if (!monitor) {
          console.warn("No monitor found");
          return;
        }

        const screenWidth = monitor.size.width;
        const screenHeight = monitor.size.height;

        let x: number;
        let y: number;

        if (snapToCorner) {
          // Snap to bottom-right corner (default Cluely behavior)
          x = screenWidth - dockWidth - margin;
          y = screenHeight - dockHeight - margin;
        } else if (cursorX !== undefined && cursorY !== undefined) {
          // Near cursor positioning (smart docking)
          // Try to place the dock near the cursor without covering it

          // Default: place to the right of cursor
          x = cursorX + margin;
          y = cursorY - dockHeight / 2;

          // Adjust if too close to right edge
          if (x + dockWidth > screenWidth - margin) {
            // Place to the left instead
            x = cursorX - dockWidth - margin;
          }

          // Adjust if too close to top edge
          if (y < margin) {
            y = margin;
          }

          // Adjust if too close to bottom edge
          if (y + dockHeight > screenHeight - margin) {
            y = screenHeight - dockHeight - margin;
          }

          // Final bounds check
          x = Math.max(margin, Math.min(x, screenWidth - dockWidth - margin));
          y = Math.max(margin, Math.min(y, screenHeight - dockHeight - margin));
        } else {
          // Default: bottom-right
          x = screenWidth - dockWidth - margin;
          y = screenHeight - dockHeight - margin;
        }

        setPosition({ x: Math.round(x), y: Math.round(y) });
        setIsDocked(true);
      } catch (error) {
        console.error("Error calculating position:", error);
      }
    },
    [dockWidth, dockHeight, margin, snapToCorner]
  );

  // Get cursor position from mouse events
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      // Store cursor position for later use
      (window as any).__lastCursorX = e.screenX;
      (window as any).__lastCursorY = e.screenY;
    };

    window.addEventListener("mousemove", handleMouseMove);
    return () => window.removeEventListener("mousemove", handleMouseMove);
  }, []);

  const dockNearCursor = useCallback(async () => {
    const cursorX = (window as any).__lastCursorX;
    const cursorY = (window as any).__lastCursorY;
    await calculateOptimalPosition(cursorX, cursorY);
  }, [calculateOptimalPosition]);

  const dockToCorner = useCallback(async () => {
    await calculateOptimalPosition();
  }, [calculateOptimalPosition]);

  const undock = useCallback(() => {
    setIsDocked(false);
  }, []);

  return {
    position,
    isDocked,
    dockNearCursor,
    dockToCorner,
    undock,
  };
}


