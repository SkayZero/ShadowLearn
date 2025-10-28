/**
 * Performance Monitoring Hook
 * Tracks and logs performance metrics for critical user interactions
 */

import { useCallback, useRef, useEffect } from 'react';
import { TOKENS } from '../lib/tokens';

interface PerfMetric {
  name: string;
  duration: number;
  timestamp: number;
  exceeded: boolean;
}

const metrics: PerfMetric[] = [];

/**
 * Hook for tracking performance marks
 * @param componentName - Name of the component being tracked
 */
export function usePerfMarks(componentName: string) {
  const markStart = useRef<number>(0);

  /**
   * Mark the start of an operation
   */
  const start = useCallback((operationName: string) => {
    markStart.current = performance.now();
    performance.mark(`shadow:${componentName}:${operationName}:start`);
  }, [componentName]);

  /**
   * Mark the end of an operation and log duration
   */
  const end = useCallback((operationName: string, maxMs?: number) => {
    const endTime = performance.now();
    const duration = endTime - markStart.current;
    
    performance.mark(`shadow:${componentName}:${operationName}:end`);
    
    try {
      performance.measure(
        `shadow:${componentName}:${operationName}`,
        `shadow:${componentName}:${operationName}:start`,
        `shadow:${componentName}:${operationName}:end`
      );
    } catch (e) {
      // Marks might not exist, ignore
    }

    // Check against target
    const exceeded = maxMs ? duration > maxMs : false;
    
    // Store metric
    const metric: PerfMetric = {
      name: `${componentName}:${operationName}`,
      duration,
      timestamp: Date.now(),
      exceeded,
    };
    
    metrics.push(metric);
    
    // Keep only last 100 metrics
    if (metrics.length > 100) {
      metrics.shift();
    }

    // Log
    const emoji = exceeded ? '⚠️' : '✅';
    const color = exceeded ? 'color: orange' : 'color: green';
    
    console.log(
      `${emoji} %c[Perf] ${metric.name}: ${duration.toFixed(2)}ms${maxMs ? ` (max: ${maxMs}ms)` : ''}`,
      color
    );

    // Auto-disable heavy features if too slow
    if (exceeded && maxMs && duration > maxMs * 2) {
      console.warn(`❌ Performance critically degraded for ${metric.name}. Consider reducing motion.`);
      localStorage.setItem('shadow:reduce-motion', 'true');
    }

    return duration;
  }, [componentName]);

  /**
   * Simple timing function
   */
  const mark = useCallback((name: string) => {
    const now = performance.now();
    console.log(`⏱️ [${componentName}] ${name}: ${now.toFixed(2)}ms`);
    return now;
  }, [componentName]);

  return { start, end, mark };
}

/**
 * Hook for guarding component render performance
 */
export function usePerfGuard(name: string, maxMs: number) {
  useEffect(() => {
    const start = performance.now();
    
    return () => {
      const duration = performance.now() - start;
      
      if (duration > maxMs) {
        console.warn(
          `⚠️ [PerfGuard] ${name} mounted for ${duration.toFixed(2)}ms (max: ${maxMs}ms)`
        );
        
        // Critical threshold
        if (duration > maxMs * 2) {
          console.error(`❌ [PerfGuard] ${name} critically slow!`);
          localStorage.setItem('shadow:reduce-motion', 'true');
        }
      }
    };
  }, [name, maxMs]);
}

/**
 * Get all collected metrics
 */
export function getAllMetrics(): PerfMetric[] {
  return [...metrics];
}

/**
 * Export metrics as CSV
 */
export function exportMetricsCSV(): string {
  const headers = ['Name', 'Duration (ms)', 'Timestamp', 'Exceeded'];
  const rows = metrics.map(m => [
    m.name,
    m.duration.toFixed(2),
    new Date(m.timestamp).toISOString(),
    m.exceeded ? 'YES' : 'NO',
  ]);

  const csv = [
    headers.join(','),
    ...rows.map(row => row.join(',')),
  ].join('\n');

  return csv;
}

/**
 * Calculate p95 for a specific metric
 */
export function calculateP95(metricName: string): number {
  const filtered = metrics.filter(m => m.name === metricName);
  
  if (filtered.length === 0) return 0;
  
  const sorted = filtered.map(m => m.duration).sort((a, b) => a - b);
  const p95Index = Math.floor(sorted.length * 0.95);
  
  return sorted[p95Index] || 0;
}

/**
 * Get performance report
 */
export function getPerformanceReport(): string {
  const uniqueMetrics = Array.from(new Set(metrics.map(m => m.name)));
  
  let report = '# Performance Report\n\n';
  report += `Generated: ${new Date().toISOString()}\n`;
  report += `Total measurements: ${metrics.length}\n\n`;
  
  report += '## Metrics Summary\n\n';
  report += '| Metric | Count | Avg (ms) | P95 (ms) | Max (ms) | Target | Status |\n';
  report += '|--------|-------|----------|----------|----------|--------|--------|\n';
  
  for (const name of uniqueMetrics) {
    const filtered = metrics.filter(m => m.name === name);
    const durations = filtered.map(m => m.duration);
    
    const count = filtered.length;
    const avg = durations.reduce((a, b) => a + b, 0) / count;
    const p95 = calculateP95(name);
    const max = Math.max(...durations);
    
    // Determine target based on metric name
    let target = '-';
    let status = '✅';
    
    if (name.includes('Dock')) {
      target = `${TOKENS.performance.bubbleToDock}ms`;
      status = p95 <= TOKENS.performance.bubbleToDock ? '✅' : '❌';
    } else if (name.includes('Toast')) {
      target = `${TOKENS.performance.toastAppear}ms`;
      status = p95 <= TOKENS.performance.toastAppear ? '✅' : '❌';
    } else if (name.includes('Pills')) {
      target = `${TOKENS.performance.pillsExpand}ms`;
      status = p95 <= TOKENS.performance.pillsExpand ? '✅' : '❌';
    }
    
    report += `| ${name} | ${count} | ${avg.toFixed(2)} | ${p95.toFixed(2)} | ${max.toFixed(2)} | ${target} | ${status} |\n`;
  }
  
  return report;
}

/**
 * Clear all metrics
 */
export function clearMetrics() {
  metrics.length = 0;
  console.log('📊 Performance metrics cleared');
}


