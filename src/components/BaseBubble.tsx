import React, { ReactNode } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

export interface BaseBubbleProps {
  /**
   * Whether the bubble is visible
   */
  isVisible: boolean;

  /**
   * Title displayed in the header
   */
  title: string | ReactNode;

  /**
   * Optional subtitle below the title
   */
  subtitle?: string | ReactNode;

  /**
   * Icon displayed before the title
   */
  icon?: string | ReactNode;

  /**
   * Main content of the bubble
   */
  children: ReactNode;

  /**
   * Footer actions (buttons, etc.)
   */
  actions?: ReactNode;

  /**
   * Callback when the close button is clicked
   */
  onClose: () => void;

  /**
   * Custom CSS class
   */
  className?: string;

  /**
   * Custom position (default: bottom right)
   */
  position?: {
    bottom?: string;
    top?: string;
    left?: string;
    right?: string;
  };

  /**
   * Custom width (default: 400px)
   */
  width?: string;

  /**
   * z-index (default: 50)
   */
  zIndex?: number;

  /**
   * Animation variant (default: fade-scale)
   */
  animation?: 'fade-scale' | 'slide-up' | 'slide-down';
}

const animations = {
  'fade-scale': {
    initial: { opacity: 0, scale: 0.9, y: 20 },
    animate: { opacity: 1, scale: 1, y: 0 },
    exit: { opacity: 0, scale: 0.9, y: 20 },
  },
  'slide-up': {
    initial: { opacity: 0, y: 50 },
    animate: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: 50 },
  },
  'slide-down': {
    initial: { opacity: 0, y: -50 },
    animate: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: -50 },
  },
};

/**
 * Base component for all bubble notifications
 *
 * Provides consistent:
 * - Animations
 * - Positioning
 * - Header/Content/Actions structure
 * - Close button
 *
 * @example
 * ```tsx
 * <BaseBubble
 *   isVisible={true}
 *   title="Suggestion"
 *   icon="💡"
 *   onClose={() => {}}
 * >
 *   <p>Your content here</p>
 * </BaseBubble>
 * ```
 */
export const BaseBubble: React.FC<BaseBubbleProps> = ({
  isVisible,
  title,
  subtitle,
  icon,
  children,
  actions,
  onClose,
  className = '',
  position = { bottom: '100px', right: '24px' },
  width = '400px',
  zIndex = 50,
  animation = 'fade-scale',
}) => {
  if (!isVisible) return null;

  const animationProps = animations[animation];

  return (
    <AnimatePresence>
      <motion.div
        {...animationProps}
        transition={{ duration: 0.3, ease: 'easeOut' }}
        className={`base-bubble ${className}`}
        style={{
          position: 'fixed',
          ...position,
          width,
          maxWidth: '90vw',
          zIndex,
          background: 'white',
          borderRadius: '16px',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.15)',
          overflow: 'hidden',
        }}
      >
        {/* Header */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-start',
            padding: '16px 20px',
            borderBottom: '1px solid rgba(0, 0, 0, 0.08)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flex: 1 }}>
            {icon && (
              <div
                style={{
                  fontSize: '24px',
                  flexShrink: 0,
                }}
              >
                {icon}
              </div>
            )}
            <div style={{ flex: 1, minWidth: 0 }}>
              <div
                style={{
                  fontSize: '16px',
                  fontWeight: 600,
                  color: '#1a1a1a',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}
              >
                {title}
              </div>
              {subtitle && (
                <div
                  style={{
                    fontSize: '13px',
                    color: '#666',
                    marginTop: '2px',
                  }}
                >
                  {subtitle}
                </div>
              )}
            </div>
          </div>
          <button
            onClick={onClose}
            style={{
              background: 'transparent',
              border: 'none',
              borderRadius: '50%',
              width: '28px',
              height: '28px',
              cursor: 'pointer',
              color: '#666',
              fontSize: '20px',
              fontWeight: 'bold',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
              transition: 'background 0.2s, color 0.2s',
              marginLeft: '8px',
            }}
            onMouseOver={(e) => {
              e.currentTarget.style.background = 'rgba(0, 0, 0, 0.05)';
              e.currentTarget.style.color = '#333';
            }}
            onMouseOut={(e) => {
              e.currentTarget.style.background = 'transparent';
              e.currentTarget.style.color = '#666';
            }}
            title="Fermer"
            aria-label="Fermer"
          >
            ✕
          </button>
        </div>

        {/* Content */}
        <div
          style={{
            padding: '20px',
            maxHeight: '60vh',
            overflowY: 'auto',
          }}
        >
          {children}
        </div>

        {/* Actions */}
        {actions && (
          <div
            style={{
              padding: '16px 20px',
              borderTop: '1px solid rgba(0, 0, 0, 0.08)',
              background: 'rgba(0, 0, 0, 0.02)',
              display: 'flex',
              gap: '8px',
              justifyContent: 'flex-end',
            }}
          >
            {actions}
          </div>
        )}
      </motion.div>
    </AnimatePresence>
  );
};

/**
 * Pre-styled button for use in bubble actions
 */
export const BubbleButton: React.FC<{
  children: ReactNode;
  onClick: () => void;
  variant?: 'primary' | 'secondary' | 'danger';
  disabled?: boolean;
}> = ({ children, onClick, variant = 'secondary', disabled = false }) => {
  const variants = {
    primary: {
      background: '#667eea',
      color: 'white',
      hoverBg: '#5568d3',
    },
    secondary: {
      background: 'rgba(0, 0, 0, 0.05)',
      color: '#333',
      hoverBg: 'rgba(0, 0, 0, 0.1)',
    },
    danger: {
      background: '#ef4444',
      color: 'white',
      hoverBg: '#dc2626',
    },
  };

  const style = variants[variant];

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        background: disabled ? '#ccc' : style.background,
        color: disabled ? '#666' : style.color,
        border: 'none',
        borderRadius: '8px',
        padding: '8px 16px',
        cursor: disabled ? 'not-allowed' : 'pointer',
        fontSize: '14px',
        fontWeight: 500,
        transition: 'background 0.2s',
        opacity: disabled ? 0.5 : 1,
      }}
      onMouseOver={(e) => {
        if (!disabled) {
          e.currentTarget.style.background = style.hoverBg;
        }
      }}
      onMouseOut={(e) => {
        if (!disabled) {
          e.currentTarget.style.background = style.background;
        }
      }}
    >
      {children}
    </button>
  );
};

export default BaseBubble;
