import { useState } from 'react';
import { createPortal } from 'react-dom';
import { invoke } from '@tauri-apps/api/core';
import './ScreenshotButton.css';

interface CaptureResult {
  data: string;  // base64
  path: string;
  size_bytes: number;
}

export function ScreenshotButton() {
  const [isOpen, setIsOpen] = useState(false);
  const [screenshotData, setScreenshotData] = useState<string | null>(null);
  const [isCapturing, setIsCapturing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [captureTime, setCaptureTime] = useState<number | null>(null);
  const [fileSize, setFileSize] = useState<number | null>(null);

  const handleCapture = async () => {
    setIsCapturing(true);
    setError(null);
    const start = performance.now();

    try {
      const result = await invoke<CaptureResult>('capture_screenshot');
      const duration = performance.now() - start;

      console.log('Screenshot captured:', {
        path: result.path,
        size: result.size_bytes,
        duration: duration + 'ms'
      });

      setScreenshotData(`data:image/jpeg;base64,${result.data}`);
      setFileSize(result.size_bytes);
      setCaptureTime(Math.round(duration));
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
    } finally {
      setIsCapturing(false);
    }
  };

  return (
    <div className="screenshot-button-container">
      <button
        className="screenshot-toggle"
        onClick={() => setIsOpen(!isOpen)}
        title="Screenshot capture"
      >
        📸
      </button>

      {isOpen && createPortal(
        <div className="screenshot-modal">
          <div className="screenshot-modal-header">
            <span className="screenshot-modal-title">📸 Screenshot</span>
          </div>

          <div className="screenshot-modal-content">
            <button
              onClick={handleCapture}
              disabled={isCapturing}
              className="capture-button"
            >
              {isCapturing ? '⏳ Capturing...' : '📸 Capture Screenshot'}
            </button>

            {error && (
              <div className="screenshot-error">
                ❌ {error}
              </div>
            )}

            {captureTime !== null && fileSize !== null && (
              <div className="screenshot-metrics">
                <div className="metric">
                  <span className="metric-label">Time:</span>
                  <span className={`metric-value ${captureTime < 800 ? 'good' : 'bad'}`}>
                    {captureTime}ms {captureTime < 800 ? '✅' : '⚠️'}
                  </span>
                </div>
                <div className="metric">
                  <span className="metric-label">Size:</span>
                  <span className={`metric-value ${fileSize < 500_000 ? 'good' : 'bad'}`}>
                    {(fileSize / 1024).toFixed(1)} KB {fileSize < 500_000 ? '✅' : '⚠️'}
                  </span>
                </div>
              </div>
            )}

            {screenshotData && (
              <div className="screenshot-preview">
                <img
                  src={screenshotData}
                  alt="Screenshot"
                  className="screenshot-image"
                />
              </div>
            )}
          </div>
        </div>,
        document.body
      )}
    </div>
  );
}
