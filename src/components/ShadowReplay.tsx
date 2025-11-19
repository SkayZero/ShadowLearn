import { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';

interface ReplayEvent {
  id: string;
  timestamp: number;
  event_type: any;
  app_name: string;
  description: string;
  metadata: any;
  screenshot_path: string | null;
}

interface ReplaySession {
  date: string;
  start_time: number;
  end_time: number;
  event_count: number;
  duration_minutes: number;
  highlights: string[];
}

interface PlaybackState {
  is_playing: boolean;
  current_index: number;
  speed: number;
  events_remaining: number;
}

interface ShadowReplayProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ShadowReplay({ isOpen, onClose }: ShadowReplayProps) {
  const [sessions, setSessions] = useState<ReplaySession[]>([]);
  const [selectedSession, setSelectedSession] = useState<ReplaySession | null>(null);
  const [events, setEvents] = useState<ReplayEvent[]>([]);
  const [playbackState, setPlaybackState] = useState<PlaybackState | null>(null);
  const [loading, setLoading] = useState(true);
  const playbackIntervalRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadSessions();
    }
    return () => {
      if (playbackIntervalRef.current) {
        clearInterval(playbackIntervalRef.current);
      }
    };
  }, [isOpen]);

  const loadSessions = async () => {
    try {
      setLoading(true);
      const sessionList = await invoke<ReplaySession[]>('get_replay_sessions');
      setSessions(sessionList);

      if (sessionList.length > 0) {
        selectSession(sessionList[0]);
      }
    } catch (error) {
      console.error('Failed to load replay sessions:', error);
    } finally {
      setLoading(false);
    }
  };

  const selectSession = async (session: ReplaySession) => {
    try {
      setSelectedSession(session);
      const eventList = await invoke<ReplayEvent[]>('get_replay_events', {
        date: session.date,
      });
      setEvents(eventList);
    } catch (error) {
      console.error('Failed to load events:', error);
    }
  };

  const startPlayback = async () => {
    try {
      await invoke('start_replay_playback');
      updatePlaybackState();

      // Start playback interval
      if (playbackIntervalRef.current) {
        clearInterval(playbackIntervalRef.current);
      }

      playbackIntervalRef.current = setInterval(async () => {
        const state = await invoke<PlaybackState>('get_playback_state');
        setPlaybackState(state);

        if (!state.is_playing) {
          if (playbackIntervalRef.current) {
            clearInterval(playbackIntervalRef.current);
          }
        }
      }, 1000 / (playbackState?.speed || 1));
    } catch (error) {
      console.error('Failed to start playback:', error);
    }
  };

  const stopPlayback = async () => {
    try {
      await invoke('stop_replay_playback');
      if (playbackIntervalRef.current) {
        clearInterval(playbackIntervalRef.current);
      }
      updatePlaybackState();
    } catch (error) {
      console.error('Failed to stop playback:', error);
    }
  };

  const setSpeed = async (speed: number) => {
    try {
      await invoke('set_replay_speed', { speed });
      updatePlaybackState();
    } catch (error) {
      console.error('Failed to set speed:', error);
    }
  };

  const updatePlaybackState = async () => {
    try {
      const state = await invoke<PlaybackState>('get_playback_state');
      setPlaybackState(state);
    } catch (error) {
      console.error('Failed to get playback state:', error);
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0, 0, 0, 0.7)',
          backdropFilter: 'blur(16px)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 70,
          padding: '24px',
        }}
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.95, y: 20 }}
          animate={{ scale: 1, y: 0 }}
          exit={{ scale: 0.95, y: 20 }}
          onClick={(e) => e.stopPropagation()}
          style={{
            maxWidth: '1000px',
            width: '100%',
            maxHeight: '90vh',
            background: 'var(--glass-bg)',
            backdropFilter: 'var(--glass-backdrop)',
            WebkitBackdropFilter: 'var(--glass-backdrop)',
            border: '1px solid var(--glass-border)',
            borderRadius: 'var(--radius-2xl)',
            boxShadow: 'var(--elev-4)',
            overflow: 'hidden',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          {/* Header */}
          <div
            style={{
              padding: '24px',
              borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
              background: 'linear-gradient(135deg, rgba(147, 51, 234, 0.2), rgba(126, 34, 206, 0.2))',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div>
                <h2
                  style={{
                    fontSize: '28px',
                    fontWeight: '700',
                    color: 'var(--text-primary)',
                    margin: 0,
                  }}
                >
                  📽️ Shadow Replay
                </h2>
                <p
                  style={{
                    fontSize: '14px',
                    color: 'var(--text-muted)',
                    margin: '4px 0 0 0',
                  }}
                >
                  Replay your day in fast-forward
                </p>
              </div>
              <button
                onClick={onClose}
                style={{
                  background: 'transparent',
                  border: 'none',
                  color: 'var(--text-muted)',
                  fontSize: '28px',
                  cursor: 'pointer',
                  padding: '0',
                  width: '36px',
                  height: '36px',
                }}
              >
                ×
              </button>
            </div>
          </div>

          {/* Content */}
          <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
            {/* Sidebar - Sessions */}
            <div
              style={{
                width: '280px',
                borderRight: '1px solid rgba(255, 255, 255, 0.1)',
                overflowY: 'auto',
                padding: '16px',
              }}
            >
              <div
                style={{
                  fontSize: '14px',
                  fontWeight: '600',
                  color: 'var(--text-primary)',
                  marginBottom: '12px',
                }}
              >
                Sessions
              </div>
              {loading ? (
                <div style={{ textAlign: 'center', padding: '40px 0', color: 'var(--text-muted)' }}>
                  Loading...
                </div>
              ) : sessions.length === 0 ? (
                <div style={{ textAlign: 'center', padding: '40px 0' }}>
                  <div style={{ fontSize: '32px', marginBottom: '8px' }}>📽️</div>
                  <div style={{ fontSize: '14px', color: 'var(--text-muted)' }}>
                    No sessions recorded
                  </div>
                </div>
              ) : (
                sessions.map((session, i) => (
                  <SessionCard
                    key={session.date}
                    session={session}
                    index={i}
                    isSelected={selectedSession?.date === session.date}
                    onSelect={() => selectSession(session)}
                  />
                ))
              )}
            </div>

            {/* Main - Timeline & Player */}
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
              {/* Timeline */}
              <div style={{ flex: 1, overflowY: 'auto', padding: '24px' }}>
                {events.length === 0 ? (
                  <div style={{ textAlign: 'center', padding: '80px 0' }}>
                    <div style={{ fontSize: '48px', marginBottom: '16px' }}>⏱️</div>
                    <div style={{ fontSize: '18px', color: 'var(--text-primary)', marginBottom: '8px' }}>
                      Select a session to view timeline
                    </div>
                    <div style={{ fontSize: '14px', color: 'var(--text-muted)' }}>
                      Your activity will appear here
                    </div>
                  </div>
                ) : (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {events.map((event, i) => (
                      <EventCard
                        key={event.id}
                        event={event}
                        index={i}
                        isPlaying={playbackState?.is_playing && playbackState.current_index === i}
                      />
                    ))}
                  </div>
                )}
              </div>

              {/* Player Controls */}
              {events.length > 0 && (
                <div
                  style={{
                    padding: '20px 24px',
                    borderTop: '1px solid rgba(255, 255, 255, 0.1)',
                    background: 'rgba(0, 0, 0, 0.3)',
                  }}
                >
                  <PlayerControls
                    playbackState={playbackState}
                    onPlay={startPlayback}
                    onStop={stopPlayback}
                    onSpeedChange={setSpeed}
                    totalEvents={events.length}
                  />
                </div>
              )}
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

// Helper Components
function SessionCard({
  session,
  index,
  isSelected,
  onSelect,
}: {
  session: ReplaySession;
  index: number;
  isSelected: boolean;
  onSelect: () => void;
}) {
  const date = new Date(session.start_time * 1000);
  const isToday = new Date().toDateString() === date.toDateString();

  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: index * 0.05 }}
      onClick={onSelect}
      style={{
        padding: '12px',
        marginBottom: '8px',
        background: isSelected ? 'rgba(147, 51, 234, 0.2)' : 'rgba(255, 255, 255, 0.05)',
        borderRadius: '8px',
        cursor: 'pointer',
        border: isSelected ? '1px solid rgba(147, 51, 234, 0.4)' : 'none',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
        <div style={{ fontSize: '14px', fontWeight: '600', color: 'var(--text-primary)' }}>
          {date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
        </div>
        {isToday && (
          <div
            style={{
              padding: '2px 6px',
              background: 'rgba(16, 185, 129, 0.2)',
              borderRadius: '4px',
              fontSize: '10px',
              fontWeight: '600',
              color: '#10b981',
            }}
          >
            TODAY
          </div>
        )}
      </div>
      <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
        {session.event_count} events • {session.duration_minutes}min
      </div>
    </motion.div>
  );
}

function EventCard({
  event,
  index,
  isPlaying,
}: {
  event: ReplayEvent;
  index: number;
  isPlaying: boolean;
}) {
  const time = new Date(event.timestamp * 1000).toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
  });

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.02 }}
      style={{
        padding: '16px',
        background: isPlaying ? 'rgba(147, 51, 234, 0.2)' : 'rgba(255, 255, 255, 0.05)',
        borderRadius: '10px',
        border: isPlaying ? '2px solid rgba(147, 51, 234, 0.6)' : 'none',
        boxShadow: isPlaying ? '0 0 20px rgba(147, 51, 234, 0.3)' : 'none',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'start', gap: '12px' }}>
        <div
          style={{
            fontSize: '11px',
            fontWeight: '600',
            color: 'var(--text-muted)',
            minWidth: '50px',
            fontFamily: 'monospace',
          }}
        >
          {time}
        </div>
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: '14px', color: 'var(--text-primary)', marginBottom: '4px' }}>
            {event.description}
          </div>
          <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>{event.app_name}</div>
        </div>
        {isPlaying && (
          <div style={{ fontSize: '20px', animation: 'pulse 1s infinite' }}>▶️</div>
        )}
      </div>
    </motion.div>
  );
}

function PlayerControls({
  playbackState,
  onPlay,
  onStop,
  onSpeedChange,
  totalEvents,
}: {
  playbackState: PlaybackState | null;
  onPlay: () => void;
  onStop: () => void;
  onSpeedChange: (speed: number) => void;
  totalEvents: number;
}) {
  const speeds = [0.5, 1, 2, 5, 10];

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '12px' }}>
        <button
          onClick={playbackState?.is_playing ? onStop : onPlay}
          style={{
            padding: '12px 24px',
            background: playbackState?.is_playing
              ? 'rgba(239, 68, 68, 0.2)'
              : 'rgba(147, 51, 234, 0.3)',
            border: 'none',
            borderRadius: '8px',
            color: 'var(--text-primary)',
            fontWeight: '600',
            cursor: 'pointer',
            fontSize: '16px',
          }}
        >
          {playbackState?.is_playing ? '⏸ Pause' : '▶️ Play'}
        </button>

        <div style={{ flex: 1 }}>
          <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginBottom: '4px' }}>
            {playbackState
              ? `${playbackState.current_index} / ${totalEvents} events`
              : `${totalEvents} events`}
          </div>
          <div
            style={{
              width: '100%',
              height: '6px',
              background: 'rgba(255, 255, 255, 0.1)',
              borderRadius: '3px',
              overflow: 'hidden',
            }}
          >
            <div
              style={{
                width: playbackState ? `${(playbackState.current_index / totalEvents) * 100}%` : '0%',
                height: '100%',
                background: 'linear-gradient(90deg, #9333ea, #7e22ce)',
                transition: 'width 0.3s ease',
              }}
            />
          </div>
        </div>

        <div style={{ display: 'flex', gap: '6px' }}>
          {speeds.map((speed) => (
            <button
              key={speed}
              onClick={() => onSpeedChange(speed)}
              style={{
                padding: '8px 12px',
                background:
                  playbackState?.speed === speed
                    ? 'rgba(147, 51, 234, 0.3)'
                    : 'rgba(255, 255, 255, 0.05)',
                border: 'none',
                borderRadius: '6px',
                color: 'var(--text-primary)',
                fontSize: '12px',
                fontWeight: '600',
                cursor: 'pointer',
              }}
            >
              {speed}x
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
