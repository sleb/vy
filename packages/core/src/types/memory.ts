/**
 * Core memory types for Vy semantic memory system
 *
 * Supports two-layer architecture:
 * - Session memory: Raw conversations during active sessions
 * - Long-term memory: Processed insights and learnings
 */

// Base types
export type MemoryId = string;
export type Timestamp = Date;
export type Embedding = number[];

/**
 * Different types of memories we can store
 * Start with 'conversation', expand to others in Phase 2
 */
export type MemoryType = 'conversation' | 'insight' | 'learning' | 'fact' | 'action_item';

/**
 * Base interface for all memory types
 */
export interface BaseMemory {
  id: MemoryId;
  type: MemoryType;
  content: string;
  timestamp: Timestamp;
  metadata: Record<string, unknown>;
  embedding?: Embedding; // Optional for now, will be populated by vector store
}

/**
 * Conversation memory - stores full conversations
 * This is our MVP implementation target
 */
export interface ConversationMemory extends BaseMemory {
  type: 'conversation';
  participants: string[];
  messageCount: number;
  duration?: number; // in milliseconds
  summary?: string; // Optional AI-generated summary
  tags?: string[];
}

/**
 * Insight memory - extracted learnings and patterns
 * Phase 2 implementation
 */
export interface InsightMemory extends BaseMemory {
  type: 'insight';
  category: 'pattern' | 'preference' | 'goal' | 'strategy' | 'relationship';
  confidence: number; // 0-1 score of how confident we are in this insight
  sourceMemories: MemoryId[]; // References to conversations that led to this insight
}

/**
 * Learning memory - specific facts or knowledge extracted
 * Phase 2 implementation
 */
export interface LearningMemory extends BaseMemory {
  type: 'learning';
  domain: string; // e.g., 'technical', 'personal', 'project'
  importance: 'low' | 'medium' | 'high';
  sourceMemories: MemoryId[];
}

/**
 * Fact memory - specific factual information
 * Phase 2 implementation
 */
export interface FactMemory extends BaseMemory {
  type: 'fact';
  factType: 'personal' | 'project' | 'preference' | 'contact' | 'other';
  verified: boolean;
  lastUpdated: Timestamp;
}

/**
 * Action item memory - extracted tasks and TODOs
 * Phase 2 implementation
 */
export interface ActionItemMemory extends BaseMemory {
  type: 'action_item';
  status: 'open' | 'in_progress' | 'completed' | 'cancelled';
  priority: 'low' | 'medium' | 'high' | 'urgent';
  dueDate?: Timestamp;
  assignee?: string;
  project?: string;
}

/**
 * Union type for all memory types
 */
export type Memory =
  | ConversationMemory
  | InsightMemory
  | LearningMemory
  | FactMemory
  | ActionItemMemory;

/**
 * Individual message within a conversation
 * For future message-level storage if needed
 */
export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Timestamp;
  metadata?: Record<string, unknown>;
}

/**
 * Full conversation structure
 * For session storage and processing
 */
export interface Conversation {
  id: string;
  messages: Message[];
  participants: string[];
  startTime: Timestamp;
  endTime?: Timestamp;
  metadata: Record<string, unknown>;
}

/**
 * Session summary created at end of conversation
 * Used for insight extraction pipeline
 */
export interface SessionSummary {
  conversationId: string;
  summary: string;
  keyPoints: string[];
  extractedInsights: string[];
  actionItems: string[];
  participants: string[];
  duration: number;
  messageCount: number;
}
