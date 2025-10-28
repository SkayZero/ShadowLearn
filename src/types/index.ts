// ============================================
// SHADOWLEARN - TYPE DEFINITIONS
// ============================================

export interface Message {
  id: number;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  screenshot?: string | null;
}

export type StatusType = 'active' | 'idle' | 'processing' | 'error';

export interface ChatWindowProps {
  messages?: Message[];
  onSendMessage?: (content: string) => void;
}

export interface MessageListProps {
  messages: Message[];
}

export interface InputFieldProps {
  onSend: (content: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

export interface StatusBadgeProps {
  status: StatusType;
}

export interface ScreenshotPreviewProps {
  imageUrl: string;
  alt?: string;
  onRemove?: () => void;
}

