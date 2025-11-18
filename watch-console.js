// Script à injecter dans la DevTools Console pour capturer tous les logs
// et les envoyer au backend Tauri pour analyse

(function() {
  const { invoke } = window.__TAURI_INTERNALS__.invoke;

  // Sauvegarder les fonctions originales
  const originalLog = console.log;
  const originalWarn = console.warn;
  const originalError = console.error;
  const originalInfo = console.info;
  const originalDebug = console.debug;

  // Fonction pour envoyer les logs au backend
  async function sendLogToBackend(level, args) {
    const message = args.map(arg => {
      if (typeof arg === 'object') {
        try {
          return JSON.stringify(arg, null, 2);
        } catch (e) {
          return String(arg);
        }
      }
      return String(arg);
    }).join(' ');

    const timestamp = new Date().toISOString();
    const logEntry = `[${timestamp}] [${level.toUpperCase()}] ${message}`;

    // Afficher dans la console aussi
    originalLog(`%c${logEntry}`, `color: ${getColorForLevel(level)}`);

    // Écrire dans un fichier via Tauri (si disponible)
    try {
      await invoke('log_to_file', {
        level,
        message,
        timestamp
      });
    } catch (e) {
      // Tauri n'est peut-être pas disponible
      originalWarn('Failed to send log to backend:', e);
    }
  }

  function getColorForLevel(level) {
    switch(level) {
      case 'error': return '#ff4444';
      case 'warn': return '#ffaa00';
      case 'info': return '#4488ff';
      case 'debug': return '#888888';
      default: return '#00ff00';
    }
  }

  // Remplacer les fonctions console
  console.log = function(...args) {
    sendLogToBackend('log', args);
  };

  console.warn = function(...args) {
    sendLogToBackend('warn', args);
  };

  console.error = function(...args) {
    sendLogToBackend('error', args);
  };

  console.info = function(...args) {
    sendLogToBackend('info', args);
  };

  console.debug = function(...args) {
    sendLogToBackend('debug', args);
  };

  originalLog('%c✅ Console monitoring actif!', 'color: #00ff00; font-weight: bold; font-size: 14px');
  originalLog('Tous les logs sont maintenant capturés et sauvegardés.');
})();
