import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useProductivity } from '../hooks/useProductivity';
import type { DayMetrics } from '../hooks/useProductivity';

interface WeeklyInsightsProps {
  isOpen: boolean;
  onClose: () => void;
}

export function WeeklyInsights({ isOpen, onClose }: WeeklyInsightsProps) {
  const { metrics, loading, refresh } = useProductivity();

  useEffect(() => {
    if (isOpen) {
      refresh();
    }
  }, [isOpen]);

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
          background: 'rgba(0, 0, 0, 0.6)',
          backdropFilter: 'blur(12px)',
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
            maxWidth: '700px',
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
              background: 'linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(5, 150, 105, 0.2))',
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
                  📊 Weekly Insights
                </h2>
                <p
                  style={{
                    fontSize: '14px',
                    color: 'var(--text-muted)',
                    margin: '4px 0 0 0',
                  }}
                >
                  {metrics?.week
                    ? `Week ${metrics.week.week_number} - Your productivity summary`
                    : 'Loading...'}
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
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  transition: 'color 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.color = 'var(--text-primary)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.color = 'var(--text-muted)';
                }}
              >
                ×
              </button>
            </div>
          </div>

          {/* Content */}
          <div style={{ flex: 1, overflowY: 'auto', padding: '24px' }}>
            {loading ? (
              <div style={{ textAlign: 'center', padding: '60px', color: 'var(--text-muted)' }}>
                <div style={{ fontSize: '18px' }}>Loading insights...</div>
              </div>
            ) : metrics?.week ? (
              <WeeklyContent week={metrics.week} />
            ) : (
              <div style={{ textAlign: 'center', padding: '60px', color: 'var(--text-muted)' }}>
                No weekly data available
              </div>
            )}
          </div>

          {/* Footer */}
          <div
            style={{
              padding: '16px 24px',
              borderTop: '1px solid rgba(255, 255, 255, 0.1)',
              display: 'flex',
              justifyContent: 'flex-end',
            }}
          >
            <button
              onClick={onClose}
              style={{
                padding: '10px 24px',
                background: 'var(--accent-primary)',
                border: 'none',
                borderRadius: '8px',
                color: 'white',
                fontWeight: '600',
                cursor: 'pointer',
                fontSize: '14px',
                transition: 'opacity 0.2s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.opacity = '0.9';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.opacity = '1';
              }}
            >
              Close
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

function WeeklyContent({ week }: { week: any }) {
  const totalHoursSaved = Math.round(week.total_time_saved / 60);
  const avgAcceptanceRate = week.total_suggestions > 0
    ? Math.round((week.total_accepted / week.total_suggestions) * 100)
    : 0;

  // Find best day
  const bestDay = week.daily_breakdown.reduce((best: DayMetrics | null, day: DayMetrics) => {
    if (!best) return day;
    const dayScore = day.acceptance_rate + (day.flow_time_minutes / 10);
    const bestScore = best.acceptance_rate + (best.flow_time_minutes / 10);
    return dayScore > bestScore ? day : best;
  }, null);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      {/* Hero Stats */}
      <motion.div
        initial={{ scale: 0.95, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        style={{
          padding: '32px',
          background: 'linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(5, 150, 105, 0.2))',
          borderRadius: 'var(--radius-xl)',
          textAlign: 'center',
        }}
      >
        <div style={{ fontSize: '48px', marginBottom: '8px' }}>🎉</div>
        <div
          style={{
            fontSize: '36px',
            fontWeight: '700',
            color: 'var(--text-primary)',
            marginBottom: '8px',
          }}
        >
          {totalHoursSaved}h saved
        </div>
        <div style={{ fontSize: '14px', color: 'var(--text-secondary)' }}>
          This week with ShadowLearn
        </div>
      </motion.div>

      {/* Key Metrics */}
      <div>
        <h3
          style={{
            fontSize: '18px',
            fontWeight: '600',
            color: 'var(--text-primary)',
            marginBottom: '16px',
          }}
        >
          Week Summary
        </h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
          <MetricBox
            icon="💡"
            label="Total Suggestions"
            value={week.total_suggestions}
            color="rgba(251, 191, 36, 0.2)"
          />
          <MetricBox
            icon="✅"
            label="Accepted"
            value={week.total_accepted}
            color="rgba(16, 185, 129, 0.2)"
          />
          <MetricBox
            icon="🎯"
            label="Acceptance Rate"
            value={`${avgAcceptanceRate}%`}
            color="rgba(99, 102, 241, 0.2)"
          />
          <MetricBox
            icon="🧘"
            label="Flow Time"
            value={`${Math.round(week.total_flow_time / 60)}h`}
            color="rgba(14, 165, 233, 0.2)"
          />
        </div>
      </div>

      {/* Best Day */}
      {bestDay && (
        <div>
          <h3
            style={{
              fontSize: '18px',
              fontWeight: '600',
              color: 'var(--text-primary)',
              marginBottom: '16px',
            }}
          >
            🏆 Best Day
          </h3>
          <motion.div
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={{ delay: 0.1 }}
            style={{
              padding: '20px',
              background: 'linear-gradient(135deg, rgba(251, 191, 36, 0.2), rgba(245, 158, 11, 0.2))',
              borderRadius: 'var(--radius-lg)',
              border: '1px solid rgba(251, 191, 36, 0.3)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
              <div>
                <div style={{ fontSize: '18px', fontWeight: '600', color: 'var(--text-primary)' }}>
                  {new Date(bestDay.date).toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' })}
                </div>
                <div style={{ fontSize: '13px', color: 'var(--text-muted)', marginTop: '4px' }}>
                  Your most productive day this week
                </div>
              </div>
              <div style={{ fontSize: '32px' }}>⭐</div>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px' }}>
              <BestDayStat label="Accepted" value={bestDay.suggestions_accepted} />
              <BestDayStat label="Rate" value={`${Math.round(bestDay.acceptance_rate)}%`} />
              <BestDayStat label="Flow" value={`${Math.round(bestDay.flow_time_minutes / 60)}h`} />
            </div>
          </motion.div>
        </div>
      )}

      {/* Daily Breakdown */}
      <div>
        <h3
          style={{
            fontSize: '18px',
            fontWeight: '600',
            color: 'var(--text-primary)',
            marginBottom: '16px',
          }}
        >
          Daily Breakdown
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          {week.daily_breakdown.map((day: DayMetrics, i: number) => (
            <DayCard key={day.date} day={day} index={i} isBest={day.date === bestDay?.date} />
          ))}
        </div>
      </div>

      {/* Week over Week */}
      {week.improvement_vs_last_week !== 0 && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          style={{
            padding: '20px',
            background: week.improvement_vs_last_week > 0
              ? 'linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(5, 150, 105, 0.2))'
              : 'linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(220, 38, 38, 0.2))',
            borderRadius: 'var(--radius-lg)',
            textAlign: 'center',
          }}
        >
          <div style={{ fontSize: '28px', marginBottom: '8px' }}>
            {week.improvement_vs_last_week > 0 ? '📈' : '📉'}
          </div>
          <div style={{ fontSize: '24px', fontWeight: '700', color: 'var(--text-primary)' }}>
            {week.improvement_vs_last_week > 0 ? '+' : ''}
            {week.improvement_vs_last_week.toFixed(1)}%
          </div>
          <div style={{ fontSize: '13px', color: 'var(--text-muted)', marginTop: '4px' }}>
            vs last week
          </div>
        </motion.div>
      )}

      {/* Achievements */}
      <div>
        <h3
          style={{
            fontSize: '18px',
            fontWeight: '600',
            color: 'var(--text-primary)',
            marginBottom: '16px',
          }}
        >
          Achievements
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {week.total_accepted >= 50 && (
            <Achievement
              icon="🏆"
              title="Power User"
              description="Accepted 50+ suggestions this week"
              color="rgba(251, 191, 36, 0.2)"
            />
          )}
          {avgAcceptanceRate >= 80 && (
            <Achievement
              icon="🎯"
              title="High Precision"
              description={`${avgAcceptanceRate}% acceptance rate`}
              color="rgba(99, 102, 241, 0.2)"
            />
          )}
          {week.total_flow_time >= 600 && (
            <Achievement
              icon="🧘"
              title="Flow Master"
              description={`${Math.round(week.total_flow_time / 60)}+ hours in flow state`}
              color="rgba(14, 165, 233, 0.2)"
            />
          )}
          {week.total_time_saved >= 180 && (
            <Achievement
              icon="⚡"
              title="Time Saver"
              description={`Saved ${Math.round(week.total_time_saved / 60)}+ hours`}
              color="rgba(16, 185, 129, 0.2)"
            />
          )}
        </div>
      </div>
    </div>
  );
}

// Helper Components
function MetricBox({
  icon,
  label,
  value,
  color,
}: {
  icon: string;
  label: string;
  value: string | number;
  color: string;
}) {
  return (
    <div
      style={{
        padding: '20px',
        background: color,
        borderRadius: 'var(--radius-lg)',
      }}
    >
      <div style={{ fontSize: '28px', marginBottom: '12px' }}>{icon}</div>
      <div style={{ fontSize: '24px', fontWeight: '700', color: 'var(--text-primary)' }}>
        {value}
      </div>
      <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginTop: '4px' }}>
        {label}
      </div>
    </div>
  );
}

function BestDayStat({ label, value }: { label: string; value: string | number }) {
  return (
    <div style={{ textAlign: 'center' }}>
      <div style={{ fontSize: '20px', fontWeight: '700', color: 'var(--text-primary)' }}>
        {value}
      </div>
      <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>
        {label}
      </div>
    </div>
  );
}

function DayCard({ day, index, isBest }: { day: DayMetrics; index: number; isBest: boolean }) {
  const date = new Date(day.date);
  const dayName = date.toLocaleDateString('en-US', { weekday: 'short' });
  const isToday = index === 0;

  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: index * 0.05 }}
      style={{
        padding: '16px',
        background: isToday
          ? 'rgba(99, 102, 241, 0.15)'
          : isBest
            ? 'rgba(251, 191, 36, 0.15)'
            : 'rgba(255, 255, 255, 0.05)',
        borderRadius: '10px',
        border: isBest ? '1px solid rgba(251, 191, 36, 0.3)' : 'none',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <div style={{ fontSize: '16px', fontWeight: '600', color: 'var(--text-primary)' }}>
            {dayName}
          </div>
          {isToday && (
            <div
              style={{
                padding: '2px 8px',
                background: 'rgba(99, 102, 241, 0.3)',
                borderRadius: '4px',
                fontSize: '10px',
                fontWeight: '600',
                color: 'var(--text-primary)',
              }}
            >
              TODAY
            </div>
          )}
          {isBest && (
            <div style={{ fontSize: '16px' }}>⭐</div>
          )}
        </div>
        <div style={{ fontSize: '13px', color: 'var(--text-muted)' }}>
          {day.suggestions_accepted}/{day.suggestions_shown}
        </div>
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '8px' }}>
        <DayMetric label="Rate" value={`${Math.round(day.acceptance_rate)}%`} />
        <DayMetric label="Saved" value={`${day.time_saved_minutes}m`} />
        <DayMetric label="Flow" value={`${Math.round(day.flow_time_minutes / 60)}h`} />
        <DayMetric label="Alerts" value={day.interruptions} />
      </div>
    </motion.div>
  );
}

function DayMetric({ label, value }: { label: string; value: string | number }) {
  return (
    <div style={{ textAlign: 'center' }}>
      <div style={{ fontSize: '14px', fontWeight: '600', color: 'var(--text-primary)' }}>
        {value}
      </div>
      <div style={{ fontSize: '10px', color: 'var(--text-muted)', marginTop: '2px' }}>
        {label}
      </div>
    </div>
  );
}

function Achievement({
  icon,
  title,
  description,
  color,
}: {
  icon: string;
  title: string;
  description: string;
  color: string;
}) {
  return (
    <div
      style={{
        padding: '16px',
        background: color,
        borderRadius: 'var(--radius-lg)',
        display: 'flex',
        alignItems: 'center',
        gap: '16px',
      }}
    >
      <div style={{ fontSize: '32px' }}>{icon}</div>
      <div>
        <div style={{ fontSize: '15px', fontWeight: '600', color: 'var(--text-primary)' }}>
          {title}
        </div>
        <div style={{ fontSize: '13px', color: 'var(--text-secondary)', marginTop: '2px' }}>
          {description}
        </div>
      </div>
    </div>
  );
}
