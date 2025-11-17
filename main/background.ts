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

          // Try to get project name from first session's cwd field
          // First extract project name from encoded directory name
          let projectName = extractProjectNameFromDirName(entry.name)

          console.log('====================================')
          console.log('📁 Processing:', entry.name)
          console.log('   Initial project name:', projectName)

          if (sessionFiles.length > 0 && claudeParser) {
            // Sort session files by modification time (newest first)
            const sessionFilesWithStats = await Promise.all(
              sessionFiles.map(async f => {
                const filePath = path.join(projectPath, f)
                const stats = await fs.stat(filePath)
                return { name: f, mtime: stats.mtimeMs }
              })
            )
            sessionFilesWithStats.sort((a, b) => b.mtime - a.mtime) // Newest first

            // Try to find a session with cwd field (check up to 10 recent sessions)
            let foundCwd = false
            const filesToCheck = sessionFilesWithStats.slice(0, 10) // Take 10 newest files

            for (const fileInfo of filesToCheck) {
              try {
                const sessionPath = path.join(projectPath, fileInfo.name)
                const summary = claudeParser.getSessionSummary(sessionPath)

                console.log(`   🔍 Checking ${fileInfo.name}`)
                console.log(`      Full summary:`, JSON.stringify(summary, null, 2))
                console.log(`      CWD value: "${summary.cwd}"`)
                console.log(`      CWD type: ${typeof summary.cwd}`)

                if (summary.cwd) {
                  projectName = extractProjectName(summary.cwd)
                  console.log('   ✅ Using cwd, final name:', projectName)
                  foundCwd = true
                  break
                } else {
                  console.log('   ⚠️  No CWD in this file, trying next...')
                }
              } catch (error) {
                console.log(`   ❌ Error reading ${fileInfo.name}:`, error.message)
              }
            }

            if (!foundCwd) {
              console.log('   ⚠️  No cwd found in any session, keeping:', projectName)
            }
          }

          console.log('   🎯 FINAL PROJECT NAME:', projectName)

          projects.push({
            name: projectName,
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

// Helper function to extract project name from encoded directory name
// Example: "-Users-name-Documents-my-project" -> "my-project"
function extractProjectNameFromDirName(dirName: string): string {
  if (!dirName.startsWith('-')) {
    return dirName
  }

  // Remove leading '-' and split by '-'
  const parts = dirName.substring(1).split('-')

  // The last part is the project folder name
  // But we need to handle cases where project name itself has dashes
  // Strategy: Find the last occurrence of common path segments (Users, Documents, etc.)
  // and take everything after that

  // Common base directories
  const commonPaths = ['Users', 'home', 'Documents', 'Desktop', 'projects', 'code', 'dev', 'workspace']

  let projectStartIdx = -1
  for (let i = parts.length - 1; i >= 0; i--) {
    if (commonPaths.includes(parts[i])) {
      projectStartIdx = i + 1
      break
    }
  }

  // If we found a common path, take everything after it
  if (projectStartIdx > 0 && projectStartIdx < parts.length) {
    return parts.slice(projectStartIdx).join('-')
  }

  // Fallback: just take the last part
  return parts[parts.length - 1] || dirName
}

// Helper function to extract project name from a path
function extractProjectName(fullPath: string): string {
  // Remove trailing slashes
  const cleanPath = fullPath.replace(/\/+$/, '')

  // Split and get the last part
  const parts = cleanPath.split('/')
  return parts[parts.length - 1] || cleanPath
}
