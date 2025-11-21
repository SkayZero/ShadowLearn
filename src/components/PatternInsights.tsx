/**
 * Pattern Insights Component
 * Phase 2.1 - Display learned workflow patterns and predictions
 */

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  usePatternInsights,
  useRepetitiveTaskDetection,
  formatActionType,
  formatDuration,
  formatConfidence,
  type WorkflowPattern,
  type Prediction,
  type RepetitiveTask,
} from '../hooks/usePatternInsights';

interface PatternInsightsProps {
  isOpen: boolean;
  onClose: () => void;
}

export function PatternInsights({ isOpen, onClose }: PatternInsightsProps) {
  const [activeTab, setActiveTab] = useState<'patterns' | 'prediction' | 'repetitions'>('patterns');

  const {
    patterns,
    prediction,
    stats,
    isLoading: patternsLoading,
    error: patternsError,
    getPrediction,
    savePatterns,
  } = usePatternInsights();

  const {
    tasks: repetitiveTasks,
    highPriorityTasks,
    isLoading: tasksLoading,
  } = useRepetitiveTaskDetection();

  if (!isOpen) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        style={{
          position: 'fixed',
          inset: 0,
          background: 'rgba(0, 0, 0, 0.5)',
          backdropFilter: 'blur(4px)',
          zIndex: 10000,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          onClick={(e) => e.stopPropagation()}
          style={{
            width: '90%',
            maxWidth: '800px',
            maxHeight: '80vh',
            background: 'var(--glass-bg)',
            backdropFilter: 'var(--glass-backdrop)',
            WebkitBackdropFilter: 'var(--glass-backdrop)',
            border: '1px solid var(--glass-border)',
            borderRadius: '16px',
            boxShadow: 'var(--glass-shadow)',
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden',
          }}
        >
          {/* Header */}
          <div
            style={{
              padding: '20px 24px',
              borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div>
              <h2
                style={{
                  fontSize: '20px',
                  fontWeight: '700',
                  color: 'var(--text-primary)',
                  margin: 0,
                }}
              >
                🧠 Pattern Insights
              </h2>
              <p
                style={{
                  fontSize: '13px',
                  color: 'var(--text-muted)',
                  margin: '4px 0 0 0',
                }}
              >
                AI-learned workflow patterns and predictions
              </p>
            </div>
            <button
              onClick={onClose}
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '8px',
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                color: 'var(--text-secondary)',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: '18px',
              }}
            >
              ×
            </button>
          </div>

          {/* Tabs */}
          <div
            style={{
              display: 'flex',
              gap: '8px',
              padding: '16px 24px',
              borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
            }}
          >
            <TabButton
              active={activeTab === 'patterns'}
              onClick={() => setActiveTab('patterns')}
              icon="🔄"
              label="Patterns"
              count={patterns.length}
            />
            <TabButton
              active={activeTab === 'prediction'}
              onClick={() => setActiveTab('prediction')}
              icon="🔮"
              label="Prediction"
            />
            <TabButton
              active={activeTab === 'repetitions'}
              onClick={() => setActiveTab('repetitions')}
              icon="🔁"
              label="Repetitions"
              count={highPriorityTasks.length}
            />
          </div>

          {/* Content */}
          <div
            style={{
              flex: 1,
              overflow: 'auto',
              padding: '24px',
            }}
          >
            {activeTab === 'patterns' && (
              <PatternsTab patterns={patterns} isLoading={patternsLoading} error={patternsError} />
            )}
            {activeTab === 'prediction' && (
              <PredictionTab
                prediction={prediction}
                onRefresh={getPrediction}
                isLoading={patternsLoading}
              />
            )}
            {activeTab === 'repetitions' && (
              <RepetitionsTab
                tasks={highPriorityTasks}
                allTasksCount={repetitiveTasks.length}
                isLoading={tasksLoading}
              />
            )}
          </div>

          {/* Footer */}
          <div
            style={{
              padding: '16px 24px',
              borderTop: '1px solid rgba(255, 255, 255, 0.1)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
              {stats && (
                <>
                  {stats.learning.total_patterns_discovered} patterns ·{' '}
                  {stats.learning.total_actions_recorded} actions ·{' '}
                  {Math.round(stats.learning.avg_pattern_confidence * 100)}% avg confidence
                </>
              )}
            </div>
            <button
              onClick={savePatterns}
              style={{
                padding: '8px 16px',
                background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-emerald))',
                border: 'none',
                borderRadius: '8px',
                color: 'white',
                fontSize: '13px',
                fontWeight: '600',
                cursor: 'pointer',
              }}
            >
              💾 Save
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

// Tab Button Component
function TabButton({
  active,
  onClick,
  icon,
  label,
  count,
}: {
  active: boolean;
  onClick: () => void;
  icon: string;
  label: string;
  count?: number;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        padding: '8px 16px',
        background: active
          ? 'linear-gradient(135deg, rgba(135, 206, 235, 0.2), rgba(16, 185, 129, 0.2))'
          : 'rgba(255, 255, 255, 0.05)',
        border: active ? '1px solid var(--accent-primary)' : '1px solid rgba(255, 255, 255, 0.1)',
        borderRadius: '8px',
        color: active ? 'var(--text-primary)' : 'var(--text-secondary)',
        fontSize: '13px',
        fontWeight: active ? '600' : '500',
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        transition: 'all 0.2s',
      }}
    >
      <span>{icon}</span>
      <span>{label}</span>
      {count !== undefined && (
        <span
          style={{
            background: active ? 'var(--accent-primary)' : 'rgba(255, 255, 255, 0.1)',
            padding: '2px 6px',
            borderRadius: '4px',
            fontSize: '11px',
            fontWeight: '700',
          }}
        >
          {count}
        </span>
      )}
    </button>
  );
}

// Patterns Tab
function PatternsTab({
  patterns,
  isLoading,
  error,
}: {
  patterns: WorkflowPattern[];
  isLoading: boolean;
  error: string | null;
}) {
  if (isLoading) {
    return <div style={{ textAlign: 'center', color: 'var(--text-muted)' }}>Loading patterns...</div>;
  }

  if (error) {
    return (
      <div style={{ textAlign: 'center', color: '#ef4444' }}>
        Error: {error}
      </div>
    );
  }

  if (patterns.length === 0) {
    return (
      <div
        style={{
          textAlign: 'center',
          padding: '40px',
          color: 'var(--text-muted)',
        }}
      >
        <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔄</div>
        <p>No patterns learned yet</p>
        <p style={{ fontSize: '12px' }}>Keep using the app to discover workflow patterns</p>
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      {patterns.map((pattern) => (
        <PatternCard key={pattern.id} pattern={pattern} />
      ))}
    </div>
  );
}

// Pattern Card
function PatternCard({ pattern }: { pattern: WorkflowPattern }) {
  return (
    <div
      style={{
        padding: '16px',
        background: 'rgba(255, 255, 255, 0.05)',
        border: '1px solid rgba(255, 255, 255, 0.1)',
        borderRadius: '12px',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '12px' }}>
        <div>
          <div style={{ fontSize: '14px', fontWeight: '600', color: 'var(--text-primary)' }}>
            {pattern.name}
          </div>
          <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px' }}>
            {pattern.occurrences} occurrences · {formatDuration(pattern.avg_duration_secs)} avg duration
          </div>
        </div>
        <div
          style={{
            fontSize: '13px',
            fontWeight: '700',
            color: pattern.confidence > 0.8 ? '#10b981' : '#f59e0b',
          }}
        >
          {formatConfidence(pattern.confidence)}
        </div>
      </div>

      <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap', marginBottom: '8px' }}>
        {pattern.sequence.map((action, idx) => (
          <span
            key={idx}
            style={{
              padding: '4px 8px',
              background: 'rgba(135, 206, 235, 0.2)',
              borderRadius: '4px',
              fontSize: '11px',
              color: 'var(--text-secondary)',
            }}
          >
            {action.app_name}
          </span>
        ))}
      </div>

      {pattern.tags.length > 0 && (
        <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
          {pattern.tags.map((tag) => (
            <span
              key={tag}
              style={{
                padding: '2px 6px',
                background: 'rgba(16, 185, 129, 0.2)',
                borderRadius: '4px',
                fontSize: '10px',
                color: '#10b981',
              }}
            >
              #{tag}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

// Prediction Tab
function PredictionTab({
  prediction,
  onRefresh,
  isLoading,
}: {
  prediction: Prediction | null;
  onRefresh: () => void;
  isLoading: boolean;
}) {
  if (isLoading) {
    return <div style={{ textAlign: 'center', color: 'var(--text-muted)' }}>Analyzing...</div>;
  }

  if (!prediction) {
    return (
      <div
        style={{
          textAlign: 'center',
          padding: '40px',
          color: 'var(--text-muted)',
        }}
      >
        <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔮</div>
        <p>No prediction available</p>
        <p style={{ fontSize: '12px', marginBottom: '16px' }}>
          Not enough context to make a prediction
        </p>
        <button
          onClick={onRefresh}
          style={{
            padding: '8px 16px',
            background: 'var(--accent-primary)',
            border: 'none',
            borderRadius: '8px',
            color: 'white',
            fontSize: '13px',
            cursor: 'pointer',
          }}
        >
          🔄 Refresh
        </button>
      </div>
    );
  }

  return (
    <div>
      <div
        style={{
          padding: '20px',
          background: 'linear-gradient(135deg, rgba(135, 206, 235, 0.2), rgba(16, 185, 129, 0.2))',
          border: '1px solid var(--accent-primary)',
          borderRadius: '12px',
          marginBottom: '16px',
        }}
      >
        <div style={{ fontSize: '16px', fontWeight: '700', color: 'var(--text-primary)', marginBottom: '8px' }}>
          🎯 Next Action Prediction
        </div>
        <div style={{ fontSize: '14px', color: 'var(--text-secondary)', marginBottom: '12px' }}>
          {formatActionType(prediction.predicted_action.action_type)} in{' '}
          <strong>{prediction.predicted_action.app_name}</strong>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>{prediction.reasoning}</div>
          <div style={{ fontSize: '14px', fontWeight: '700', color: '#10b981' }}>
            {formatConfidence(prediction.confidence)}
          </div>
        </div>
      </div>

      {prediction.alternative_predictions.length > 0 && (
        <div>
          <div style={{ fontSize: '13px', fontWeight: '600', marginBottom: '8px', color: 'var(--text-muted)' }}>
            Alternative Predictions
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {prediction.alternative_predictions.map((alt, idx) => (
              <div
                key={idx}
                style={{
                  padding: '12px',
                  background: 'rgba(255, 255, 255, 0.05)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '8px',
                  display: 'flex',
                  justifyContent: 'space-between',
                }}
              >
                <span style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                  {formatActionType(alt.action.action_type)} in {alt.action.app_name}
                </span>
                <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
                  {formatConfidence(alt.confidence)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      <button
        onClick={onRefresh}
        style={{
          width: '100%',
          marginTop: '16px',
          padding: '10px',
          background: 'rgba(255, 255, 255, 0.05)',
          border: '1px solid rgba(255, 255, 255, 0.1)',
          borderRadius: '8px',
          color: 'var(--text-secondary)',
          fontSize: '13px',
          cursor: 'pointer',
        }}
      >
        🔄 Refresh Prediction
      </button>
    </div>
  );
}

// Repetitions Tab
function RepetitionsTab({
  tasks,
  allTasksCount,
  isLoading,
}: {
  tasks: RepetitiveTask[];
  allTasksCount: number;
  isLoading: boolean;
}) {
  if (isLoading) {
    return <div style={{ textAlign: 'center', color: 'var(--text-muted)' }}>Loading tasks...</div>;
  }

  if (tasks.length === 0) {
    return (
      <div
        style={{
          textAlign: 'center',
          padding: '40px',
          color: 'var(--text-muted)',
        }}
      >
        <div style={{ fontSize: '48px', marginBottom: '16px' }}>✅</div>
        <p>No repetitive tasks detected</p>
        <p style={{ fontSize: '12px' }}>Keep working to identify automation opportunities</p>
      </div>
    );
  }

  return (
    <div>
      <div style={{ marginBottom: '16px', fontSize: '12px', color: 'var(--text-muted)' }}>
        Showing {tasks.length} high-priority tasks (out of {allTasksCount} total)
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        {tasks.map((task) => (
          <RepetitiveTaskCard key={task.id} task={task} />
        ))}
      </div>
    </div>
  );
}

// Repetitive Task Card
function RepetitiveTaskCard({ task }: { task: RepetitiveTask }) {
  return (
    <div
      style={{
        padding: '16px',
        background: 'rgba(251, 191, 36, 0.1)',
        border: '1px solid rgba(251, 191, 36, 0.3)',
        borderRadius: '12px',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
        <div style={{ fontSize: '14px', fontWeight: '600', color: 'var(--text-primary)' }}>
          {task.name}
        </div>
        <div
          style={{
            fontSize: '12px',
            fontWeight: '700',
            color: task.automation_potential > 0.7 ? '#10b981' : '#f59e0b',
          }}
        >
          {Math.round(task.automation_potential * 100)}% automatable
        </div>
      </div>

      <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginBottom: '8px' }}>
        {task.repetitions} repetitions · {Math.round(task.time_wasted_mins)} mins wasted
      </div>

      <div
        style={{
          padding: '8px',
          background: 'rgba(16, 185, 129, 0.1)',
          borderRadius: '6px',
          fontSize: '12px',
          color: '#10b981',
        }}
      >
        💡 {task.automation_suggestion}
      </div>
    </div>
  );
}
