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

// ============================================
// OPPORTUNITY TYPES (Phase 3A)
// ============================================

export type OpportunityType = 'refacto' | 'debug' | 'learn' | 'tip';

export type OpportunityStatus = 'pending' | 'viewed' | 'actioned' | 'ignored';

export interface OpportunityContext {
  app: string;
  file?: string;
  line?: number;
  codeSnippet?: string;
}

export interface OpportunityAction {
  id: string;
  label: string;
  icon: string;
  type: 'discuss' | 'view' | 'ignore' | 'custom';
}

export interface Opportunity {
  id: string;
  title: string;
  description: string;
  context: OpportunityContext;
  type: OpportunityType;
  confidence: number;
  timestamp: number;
  status: OpportunityStatus;
  actions: OpportunityAction[];
}

