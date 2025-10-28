/**
 * Cluely Sound Manager
 * Gestion des effets sonores subtils et professionnels
 */

interface SoundConfig {
  name: string;
  url: string;
  volume: number; // 0-1
  duration?: number; // in seconds
}

const SOUNDS: Record<string, SoundConfig> = {
  'ui-ready': {
    name: 'UI Ready',
    url: '/sounds/ui-ready.mp3', // You'll need to add these files
    volume: 0.25,
    duration: 0.2,
  },
  'toast-in': {
    name: 'Toast In',
    url: '/sounds/toast-in.mp3',
    volume: 0.25,
    duration: 0.15,
  },
  'toast-out': {
    name: 'Toast Out',
    url: '/sounds/toast-out.mp3',
    volume: 0.25,
    duration: 0.15,
  },
  'dock-open': {
    name: 'Dock Open',
    url: '/sounds/dock-open.mp3',
    volume: 0.25,
    duration: 0.3,
  },
  'dock-close': {
    name: 'Dock Close',
    url: '/sounds/dock-close.mp3',
    volume: 0.25,
    duration: 0.2,
  },
  'success': {
    name: 'Success',
    url: '/sounds/success.mp3',
    volume: 0.25,
    duration: 0.4,
  },
  'error': {
    name: 'Error',
    url: '/sounds/error.mp3',
    volume: 0.25,
    duration: 0.5,
  },
  'click': {
    name: 'Click',
    url: '/sounds/click.mp3',
    volume: 0.15,
    duration: 0.1,
  },
};

class SoundManager {
  private enabled: boolean = true;
  private globalVolume: number = 0.25;
  private sounds: Map<string, HTMLAudioElement> = new Map();

  constructor() {
    this.loadSounds();
  }

  /**
   * Load all sound files
   */
  private loadSounds(): void {
    Object.entries(SOUNDS).forEach(([key, config]) => {
      const audio = new Audio(config.url);
      audio.preload = 'auto';
      audio.volume = this.globalVolume;
      this.sounds.set(key, audio);
    });
  }

  /**
   * Enable/disable sounds
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
  }

  /**
   * Set global volume (0-1)
   */
  setVolume(volume: number): void {
    this.globalVolume = Math.max(0, Math.min(1, volume));
    this.sounds.forEach((audio) => {
      audio.volume = this.globalVolume;
    });
  }

  /**
   * Play a sound
   */
  play(soundName: string): void {
    if (!this.enabled) return;

    const sound = this.sounds.get(soundName);
    if (!sound) {
      console.warn(`Sound "${soundName}" not found`);
      return;
    }

    try {
      // Clone and play to allow multiple instances
      const audio = sound.cloneNode() as HTMLAudioElement;
      audio.volume = this.globalVolume;
      audio.play().catch((err) => {
        console.warn(`Failed to play sound "${soundName}":`, err);
      });
    } catch (err) {
      console.warn(`Error playing sound "${soundName}":`, err);
    }
  }

  /**
   * Stop all sounds
   */
  stopAll(): void {
    this.sounds.forEach((audio) => {
      audio.pause();
      audio.currentTime = 0;
    });
  }
}

// Singleton instance
export const soundManager = new SoundManager();

// Listen to Tauri events for sound triggers
if (typeof window !== 'undefined') {
  import('@tauri-apps/api/event').then(({ listen }) => {
    listen('shadow:sound:play', (event: any) => {
      const soundName = event.payload?.sound;
      if (soundName) {
        soundManager.play(soundName);
      }
    });
  });
}

export default soundManager;




