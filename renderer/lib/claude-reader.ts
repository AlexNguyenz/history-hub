/**
 * Claude Code History Reader
 * Wrapper around Rust native module for reading Claude conversation history
 */

// Declare window.ipc type
declare global {
  interface Window {
    ipc: {
      send(channel: string, value: unknown): void;
      invoke(channel: string, ...args: unknown[]): Promise<any>;
      on(channel: string, callback: (...args: unknown[]) => void): () => void;
    };
  }
}

// ============================================
// TYPE DEFINITIONS
// ============================================

export interface ClaudeMessage {
  messageId: string;
  sessionId: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
  parentId?: string;
  model?: string;
  stopReason?: string;
  inputTokens?: number;
  outputTokens?: number;
}

export interface ClaudeSession {
  sessionId: string;
  filePath: string;
  messageCount: number;
  userMessageCount: number;
  assistantMessageCount: number;
  firstTimestamp?: string;
  lastTimestamp?: string;
}

export interface ClaudeProject {
  name: string;
  path: string;
  sessionCount: number;
  sessions?: ClaudeSession[];
}

// ============================================
// CONSTANTS
// ============================================
// (Not needed anymore - handled in main process)

// ============================================
// CORE FUNCTIONS
// ============================================

/**
 * Parse a Claude session file and return all messages
 */
export async function parseSession(filePath: string): Promise<ClaudeMessage[]> {
  try {
    const messages = await window.ipc.invoke('parse-session', filePath) as ClaudeMessage[];
    return messages;
  } catch (error) {
    console.error('Error parsing session:', error);
    throw new Error(`Failed to parse session: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Get session summary (metadata only, faster than parsing all messages)
 */
export async function getSessionSummary(filePath: string): Promise<ClaudeSession> {
  try {
    const summary = await window.ipc.invoke('get-session-summary', filePath) as ClaudeSession;
    return summary;
  } catch (error) {
    console.error('Error getting session summary:', error);
    throw new Error(`Failed to get session summary: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Get all Claude projects from ~/.claude/projects
 */
export async function getAllProjects(): Promise<ClaudeProject[]> {
  try {
    const projects = await window.ipc.invoke('get-all-projects') as ClaudeProject[];
    return projects;
  } catch (error) {
    console.error('Error getting projects:', error);
    throw new Error(`Failed to get projects: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Get all sessions for a specific project
 */
export async function getProjectSessions(projectPath: string): Promise<ClaudeSession[]> {
  try {
    const sessions = await window.ipc.invoke('get-project-sessions', projectPath) as ClaudeSession[];
    return sessions;
  } catch (error) {
    console.error('Error getting project sessions:', error);
    throw new Error(`Failed to get project sessions: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Get full project with sessions
 */
export async function getProjectWithSessions(projectPath: string): Promise<ClaudeProject> {
  const sessions = await getProjectSessions(projectPath);

  // Extract project name from path
  const pathParts = projectPath.split('/');
  const dirName = pathParts[pathParts.length - 1];
  const projectName = decodeProjectName(dirName);

  return {
    name: projectName,
    path: projectPath,
    sessionCount: sessions.length,
    sessions,
  };
}

// ============================================
// UTILITY FUNCTIONS
// ============================================

/**
 * Decode project name from directory name
 * Example: "-Users-name-Documents-project" → "/Users/name/Documents/project"
 */
function decodeProjectName(dirName: string): string {
  if (dirName.startsWith('-')) {
    return '/' + dirName.substring(1).replace(/-/g, '/');
  }
  return dirName;
}

/**
 * Format timestamp to readable string
 */
export function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleString();
}

/**
 * Calculate time ago
 */
export function timeAgo(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  if (seconds < 2592000) return `${Math.floor(seconds / 86400)}d ago`;
  return date.toLocaleDateString();
}

/**
 * Format token count
 */
export function formatTokens(count: number): string {
  if (count < 1000) return count.toString();
  if (count < 1000000) return `${(count / 1000).toFixed(1)}k`;
  return `${(count / 1000000).toFixed(1)}M`;
}
