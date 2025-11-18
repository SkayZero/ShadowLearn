import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { TriggerBubble } from '../TriggerBubble';
import type { Context } from '../../hooks/useTrigger';

describe('TriggerBubble', () => {
  const mockContext: Context = {
    id: 'ctx_123',
    app: {
      name: 'Cursor',
      bundle_id: 'com.todesktop.230313mzl4w4u92',
      window_title: 'App.tsx - ShadowLearn',
    },
    clipboard: 'const test = "hello";',
    idle_seconds: 15.5,
    timestamp: Date.now(),
    capture_duration_ms: 25,
  };

  it('should not render when context is null', () => {
    render(
      <TriggerBubble
        context={null}
        isVisible={true}
        onHide={vi.fn()}
        onUserInteraction={vi.fn()}
      />
    );

    expect(screen.queryByText('💡 Besoin d\'aide ?')).not.toBeInTheDocument();
  });

  it('should not render when isVisible is false', () => {
    render(
      <TriggerBubble
        context={mockContext}
        isVisible={false}
        onHide={vi.fn()}
        onUserInteraction={vi.fn()}
      />
    );

    expect(screen.queryByText('💡 Besoin d\'aide ?')).not.toBeInTheDocument();
  });

  it('should render when context is provided and isVisible is true', () => {
    render(
      <TriggerBubble
        context={mockContext}
        isVisible={true}
        onHide={vi.fn()}
        onUserInteraction={vi.fn()}
      />
    );

    expect(screen.getByText('💡 Besoin d\'aide ?')).toBeInTheDocument();
    expect(screen.getByText('Cursor')).toBeInTheDocument();
    expect(screen.getByText('App.tsx - ShadowLearn')).toBeInTheDocument();
  });

  it('should display clipboard content when available', () => {
    render(
      <TriggerBubble
        context={mockContext}
        isVisible={true}
        onHide={vi.fn()}
        onUserInteraction={vi.fn()}
      />
    );

    expect(screen.getByText('📋 Clipboard récent')).toBeInTheDocument();
    expect(screen.getByText('const test = "hello";')).toBeInTheDocument();
  });

  it('should truncate long clipboard content', () => {
    const longClipboard = 'a'.repeat(150);
    const contextWithLongClipboard = {
      ...mockContext,
      clipboard: longClipboard,
    };

    render(
      <TriggerBubble
        context={contextWithLongClipboard}
        isVisible={true}
        onHide={vi.fn()}
        onUserInteraction={vi.fn()}
      />
    );

    const displayedText = screen.getByText(/^a+\.\.\.$/);
    expect(displayedText.textContent?.length).toBeLessThan(longClipboard.length);
  });

  it('should display idle seconds', () => {
    render(
      <TriggerBubble
        context={mockContext}
        isVisible={true}
        onHide={vi.fn()}
        onUserInteraction={vi.fn()}
      />
    );

    expect(screen.getByText('⏱️ Temps d\'inactivité')).toBeInTheDocument();
    expect(screen.getByText('16 secondes')).toBeInTheDocument(); // Math.round(15.5) = 16
  });

  it('should display capture duration', () => {
    render(
      <TriggerBubble
        context={mockContext}
        isVisible={true}
        onHide={vi.fn()}
        onUserInteraction={vi.fn()}
      />
    );

    expect(screen.getByText('⚡ Performance')).toBeInTheDocument();
    expect(screen.getByText('Capture: 25ms')).toBeInTheDocument();
  });

  it('should call onUserInteraction when primary button is clicked', () => {
    const onUserInteraction = vi.fn();

    render(
      <TriggerBubble
        context={mockContext}
        isVisible={true}
        onHide={vi.fn()}
        onUserInteraction={onUserInteraction}
      />
    );

    const primaryButton = screen.getByText('✅ Afficher une suggestion');
    fireEvent.click(primaryButton);

    expect(onUserInteraction).toHaveBeenCalledTimes(1);
  });

  it('should call onHide when close button is clicked', () => {
    const onHide = vi.fn();

    render(
      <TriggerBubble
        context={mockContext}
        isVisible={true}
        onHide={onHide}
        onUserInteraction={vi.fn()}
      />
    );

    const closeButton = screen.getByTitle('Fermer');
    fireEvent.click(closeButton);

    expect(onHide).toHaveBeenCalledTimes(1);
  });

  it('should call onHide when secondary button is clicked', () => {
    const onHide = vi.fn();

    render(
      <TriggerBubble
        context={mockContext}
        isVisible={true}
        onHide={onHide}
        onUserInteraction={vi.fn()}
      />
    );

    const secondaryButton = screen.getByText('❌ Plus tard');
    fireEvent.click(secondaryButton);

    expect(onHide).toHaveBeenCalledTimes(1);
  });

  it('should not display clipboard section when clipboard is null', () => {
    const contextWithoutClipboard = {
      ...mockContext,
      clipboard: null,
    };

    render(
      <TriggerBubble
        context={contextWithoutClipboard}
        isVisible={true}
        onHide={vi.fn()}
        onUserInteraction={vi.fn()}
      />
    );

    expect(screen.queryByText('📋 Clipboard récent')).not.toBeInTheDocument();
  });
});
