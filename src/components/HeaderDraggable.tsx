import React, { useRef, useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface HeaderDraggableProps {
  title: string;
  showClose?: boolean;
  showMinimize?: boolean;
  onClose?: () => void;
  onMinimize?: () => void;
  children?: React.ReactNode;
  rightContent?: React.ReactNode; // For snooze menu, badges, etc.
}

const HeaderDraggable: React.FC<HeaderDraggableProps> = ({
  title,
  showClose = true,
  showMinimize = true,
  onClose,
  onMinimize,
  children,
  rightContent,
}) => {
  const headerRef = useRef<HTMLDivElement>(null);
  const [isClosing, setIsClosing] = useState(false);

  useEffect(() => {
    const header = headerRef.current;
    if (!header) return;

    // Set drag region for Tauri v2 - always active
    header.setAttribute('data-tauri-drag-region', 'true');
    
    // Cleanup on unmount
    return () => {
      header.removeAttribute('data-tauri-drag-region');
    };
  }, []);

  const handleClose = async () => {
    if (isClosing) return; // Prevent double clicks
    
    setIsClosing(true);
    console.log('Close button clicked!'); // Debug log
    
    try {
      if (onClose) {
        onClose();
      } else {
        const window = getCurrentWindow();
        await window.hide();
        console.log('Window hidden successfully'); // Debug log
      }
    } catch (error) {
      console.error('Failed to hide window:', error);
    } finally {
      setIsClosing(false);
    }
  };

  const handleMinimize = async () => {
    try {
      if (onMinimize) {
        onMinimize();
      } else {
        const window = getCurrentWindow();
        await window.minimize();
        console.log('Window minimized successfully');
      }
    } catch (error) {
      console.error('Failed to minimize window:', error);
    }
  };

  return (
    <header ref={headerRef} className="sl-header">
      <div className="sl-header-content">
        {children}
        <span className="sl-header-title">{title}</span>
      </div>

      <div className="sl-header-right">
        {rightContent}
        {showMinimize && (
          <button
            className="sl-header-minimize"
            onClick={handleMinimize}
            aria-label="Minimize window"
            title="Minimiser"
          >
            −
          </button>
        )}
        {showClose && (
          <button
            className="sl-header-close"
            onClick={handleClose}
            disabled={isClosing}
            style={{
              opacity: isClosing ? 0.5 : 1,
              pointerEvents: 'auto',
              position: 'relative',
              zIndex: 1000,
            }}
            aria-label="Close window"
            title="Fermer"
          >
            ×
          </button>
        )}
      </div>
    </header>
  );
};

export default HeaderDraggable;