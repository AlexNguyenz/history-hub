module.exports = {
  webpack: (defaultConfig, env) => {
    // For electron main process, add 'electron' to externals
    // (it's in devDependencies, so nextron doesn't auto-externalize it)
    if (defaultConfig.target === 'electron-main') {
      if (!defaultConfig.externals.includes('electron')) {
        defaultConfig.externals.push('electron')
      }
    }
    return defaultConfig
  },
}
