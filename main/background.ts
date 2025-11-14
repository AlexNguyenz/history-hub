import path from 'path'
import { app, ipcMain } from 'electron'
import serve from 'electron-serve'
import { createWindow } from './helpers'
import os from 'os'
import fs from 'fs/promises'

const isProd = process.env.NODE_ENV === 'production'

if (isProd) {
  serve({ directory: 'app' })
}

// Will be loaded after app is ready
let claudeParser: any = null

;(async () => {
  await app.whenReady()

  if (!isProd) {
    app.setPath('userData', `${app.getPath('userData')} (development)`)
  }

  // Load native module after app is ready
  claudeParser = require('claude-parser')

  // Register IPC handlers
  registerIPCHandlers()

  const mainWindow = createWindow('main', {
    width: 1000,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
    },
  })

  if (isProd) {
    await mainWindow.loadURL('app://./home')
  } else {
    const port = process.argv[2]
    await mainWindow.loadURL(`http://localhost:${port}/home`)
    mainWindow.webContents.openDevTools()
  }
})()

app.on('window-all-closed', () => {
  app.quit()
})

ipcMain.on('message', async (event, arg) => {
  event.reply('message', `${arg} World!`)
})

// ============================================
// IPC HANDLERS FOR CLAUDE PARSER
// ============================================

const CLAUDE_DIR = path.join(os.homedir(), '.claude', 'projects')

function registerIPCHandlers() {
  // Get all Claude projects
  ipcMain.handle('get-all-projects', async () => {
    try {
      // Check if directory exists
      try {
        await fs.access(CLAUDE_DIR)
      } catch {
        return []
      }

      const entries = await fs.readdir(CLAUDE_DIR, { withFileTypes: true })
      const projects = []

      for (const entry of entries) {
        if (entry.isDirectory()) {
          const projectPath = path.join(CLAUDE_DIR, entry.name)
          const files = await fs.readdir(projectPath)
          const sessionFiles = files.filter(f => f.endsWith('.jsonl'))

          projects.push({
            name: decodeProjectName(entry.name),
            path: projectPath,
            sessionCount: sessionFiles.length,
          })
        }
      }

      return projects
    } catch (error) {
      console.error('Error getting projects:', error)
      throw error
    }
  })

  // Get sessions for a project
  ipcMain.handle('get-project-sessions', async (_event, projectPath: string) => {
    try {
      if (!claudeParser) {
        throw new Error('Claude parser not initialized')
      }

      const files = await fs.readdir(projectPath)
      const sessionFiles = files.filter(f => f.endsWith('.jsonl'))

      const sessions = []

      for (const file of sessionFiles) {
        const filePath = path.join(projectPath, file)
        try {
          const summary = claudeParser.getSessionSummary(filePath)
          sessions.push(summary)
        } catch (error) {
          console.error(`Error reading session ${file}:`, error)
        }
      }

      // Sort by last timestamp (newest first)
      sessions.sort((a, b) => {
        if (!a.lastTimestamp || !b.lastTimestamp) return 0
        return new Date(b.lastTimestamp).getTime() - new Date(a.lastTimestamp).getTime()
      })

      return sessions
    } catch (error) {
      console.error('Error getting project sessions:', error)
      throw error
    }
  })

  // Parse a session file
  ipcMain.handle('parse-session', async (_event, filePath: string) => {
    try {
      if (!claudeParser) {
        throw new Error('Claude parser not initialized')
      }
      const messages = claudeParser.parseClaudeSession(filePath)
      return messages
    } catch (error) {
      console.error('Error parsing session:', error)
      throw error
    }
  })

  // Get session summary
  ipcMain.handle('get-session-summary', async (_event, filePath: string) => {
    try {
      if (!claudeParser) {
        throw new Error('Claude parser not initialized')
      }
      const summary = claudeParser.getSessionSummary(filePath)
      return summary
    } catch (error) {
      console.error('Error getting session summary:', error)
      throw error
    }
  })
}

// Helper function
function decodeProjectName(dirName: string): string {
  if (dirName.startsWith('-')) {
    return '/' + dirName.substring(1).replace(/-/g, '/')
  }
  return dirName
}
