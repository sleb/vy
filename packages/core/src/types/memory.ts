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
export type MemoryType =
  | "conversation"
  | "insight"
  | "learning"
  | "fact"
  | "action_item";

/**
 * Conversation-specific data
 */
export interface ConversationData {
  participants: string[];
  messageCount: number;
  duration?: number; // in milliseconds
  summary?: string; // Optional AI-generated summary
  tags?: string[];
}

/**
 * Insight-specific data
 */
export interface InsightData {
  category: "pattern" | "preference" | "goal" | "strategy" | "relationship";
  confidence: number; // 0-1 score of how confident we are in this insight
  sourceMemories: MemoryId[]; // References to conversations that led to this insight
}

/**
 * Learning-specific data
 */
export interface LearningData {
  domain: string; // e.g., 'technical', 'personal', 'project'
  importance: "low" | "medium" | "high";
  sourceMemories: MemoryId[];
}

/**
 * Fact-specific data
 */
export interface FactData {
  factType: "personal" | "project" | "preference" | "contact" | "other";
  verified: boolean;
  lastUpdated: Timestamp;
}

/**
 * Action item-specific data
 */
export interface ActionItemData {
  status: "open" | "in_progress" | "completed" | "cancelled";
  priority: "low" | "medium" | "high" | "urgent";
  dueDate?: Timestamp;
  assignee?: string;
  project?: string;
}

/**
 * Memory using composition - flexible and extensible
 * Type-specific data is optional, allowing partial memories during deserialization
 */
export interface Memory {
  id: MemoryId;
  type: MemoryType;
  content: string;
  timestamp: Timestamp;
  metadata: Record<string, unknown>;
  embedding?: Embedding; // Optional for now, will be populated by vector store

  // Type-specific data (composition approach)
  conversationData?: ConversationData;
  insightData?: InsightData;
  learningData?: LearningData;
  factData?: FactData;
  actionItemData?: ActionItemData;
}

/**
 * Legacy type aliases for backward compatibility
 * These can be gradually phased out
 */
export type ConversationMemory = Memory & {
  type: "conversation";
  conversationData: ConversationData;
};
export type InsightMemory = Memory & {
  type: "insight";
  insightData: InsightData;
};
export type LearningMemory = Memory & {
  type: "learning";
  learningData: LearningData;
};
export type FactMemory = Memory & { type: "fact"; factData: FactData };
export type ActionItemMemory = Memory & {
  type: "action_item";
  actionItemData: ActionItemData;
};

/**
 * Individual message within a conversation
 * For future message-level storage if needed
 */
export interface Message {
  id: string;
  role: "user" | "assistant" | "system";
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
