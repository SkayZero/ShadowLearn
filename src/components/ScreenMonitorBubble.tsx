import React, { useEffect } from 'react';
import { useScreenSuggestion } from '../hooks/useScreenMonitor';
import { BaseBubble, BubbleButton } from './BaseBubble';

interface ScreenMonitorBubbleProps {
  /**
   * Auto-dismiss after this many seconds (0 = never auto-dismiss)
   * @default 30
   */
  autoDismissSeconds?: number;

  /**
   * Custom CSS class
   */
  className?: string;
}

/**
 * Bubble component for displaying screen monitoring suggestions
 *
 * Automatically appears when a screen change with Claude Vision analysis is detected.
 * Integrates seamlessly with ShadowLearn's UI.
 *
 * @example
 * ```tsx
 * <ScreenMonitorBubble autoDismissSeconds={30} />
 * ```
 */
export const ScreenMonitorBubble: React.FC<ScreenMonitorBubbleProps> = ({
  autoDismissSeconds = 30,
  className = '',
}) => {
  const { suggestion, dismiss } = useScreenSuggestion();

  // Auto-dismiss after timeout
  useEffect(() => {
    if (suggestion && autoDismissSeconds > 0) {
      const timer = setTimeout(() => {
        dismiss();
      }, autoDismissSeconds * 1000);

      return () => clearTimeout(timer);
    }
  }, [suggestion, autoDismissSeconds, dismiss]);

  const handleOpenChat = () => {
    // TODO: Integrate with chat window
    console.log('Opening chat with suggestion:', suggestion?.text);
    dismiss();
  };

  return (
    <BaseBubble
      isVisible={!!suggestion}
      title="Claude Vision Suggestion"
      icon="🤖"
      onClose={dismiss}
      className={className}
      position={{ bottom: '120px', right: '24px' }}
      zIndex={100}
      actions={
        <>
          <BubbleButton onClick={dismiss} variant="secondary">
            Ignorer
          </BubbleButton>
          <BubbleButton onClick={handleOpenChat} variant="primary">
            Ouvrir le Chat
          </BubbleButton>
        </>
      }
    >
      <div
        style={{
          fontSize: '15px',
          lineHeight: '1.6',
          color: '#333',
        }}
      >
        {suggestion?.text}
      </div>

      {autoDismissSeconds > 0 && (
        <div
          style={{
            marginTop: '16px',
            fontSize: '12px',
            color: '#999',
            textAlign: 'center',
          }}
        >
          Auto-dismiss dans {autoDismissSeconds}s
        </div>
      )}
    </BaseBubble>
  );
};

export default ScreenMonitorBubble;
