import React, { useState, useEffect } from 'react';
import Head from 'next/head';
import { getAllProjects, ClaudeProject } from '../lib/claude-reader';

export default function HomePage() {
  const [projects, setProjects] = useState<ClaudeProject[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadProjects();
  }, []);

  async function loadProjects() {
    try {
      setLoading(true);
      setError(null);
      const allProjects = await getAllProjects();
      setProjects(allProjects);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load projects');
      console.error('Error loading projects:', err);
    } finally {
      setLoading(false);
    }
  }

  return (
    <React.Fragment>
      <Head>
        <title>Claude Code History Hub</title>
      </Head>

      <div className="min-h-screen bg-gray-50 p-8">
        <div className="max-w-4xl mx-auto">
          {/* Header */}
          <div className="mb-8">
            <h1 className="text-4xl font-bold text-gray-900 mb-2">
              🦀 Claude Code History Hub
            </h1>
            <p className="text-gray-600">
              Browse and explore your Claude Code conversation history
            </p>
          </div>

          {/* Status */}
          {loading && (
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-4">
              <p className="text-blue-800">⏳ Loading projects...</p>
            </div>
          )}

          {error && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4">
              <p className="text-red-800">❌ Error: {error}</p>
              <button
                onClick={loadProjects}
                className="mt-2 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
              >
                Retry
              </button>
            </div>
          )}

          {/* Projects List */}
          {!loading && !error && (
            <div>
              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-2xl font-semibold text-gray-800">
                  Projects ({projects.length})
                </h2>
                <button
                  onClick={loadProjects}
                  className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                >
                  🔄 Refresh
                </button>
              </div>

              {projects.length === 0 ? (
                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6 text-center">
                  <p className="text-yellow-800 text-lg">
                    📭 No Claude Code projects found
                  </p>
                  <p className="text-yellow-600 text-sm mt-2">
                    Make sure you have used Claude Code CLI before
                  </p>
                </div>
              ) : (
                <div className="space-y-4">
                  {projects.map((project, idx) => (
                    <div
                      key={idx}
                      className="bg-white border border-gray-200 rounded-lg p-6 hover:shadow-lg transition-shadow cursor-pointer"
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <h3 className="text-lg font-semibold text-gray-900 mb-2">
                            📁 {project.name}
                          </h3>
                          <p className="text-sm text-gray-500 mb-2">
                            {project.path}
                          </p>
                          <div className="flex gap-4 text-sm">
                            <span className="text-gray-600">
                              💬 {project.sessionCount} sessions
                            </span>
                          </div>
                        </div>
                        <button className="px-4 py-2 bg-gray-100 text-gray-700 rounded hover:bg-gray-200">
                          View →
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Footer */}
          <div className="mt-8 pt-8 border-t border-gray-200 text-center text-sm text-gray-500">
            <p>Powered by Rust 🦀 + Next.js ⚡ + Electron 🖥️</p>
            <p className="mt-1">
              Reading from: <code className="bg-gray-100 px-2 py-1 rounded">~/.claude/projects</code>
            </p>
          </div>
        </div>
      </div>
    </React.Fragment>
  );
}
