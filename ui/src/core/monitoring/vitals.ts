/**
 * Web Vitals Monitoring
 * Track Core Web Vitals and other performance metrics
 */

import type { Metric } from "web-vitals";
import { onCLS, onFID, onFCP, onLCP, onTTFB, onINP } from "web-vitals";
import { metricsCollector } from "./metrics";
import { logger } from "../utils/monitoring/logger";

/**
 * Initialize Web Vitals monitoring
 */
export function initWebVitals(): void {
  // Core Web Vitals
  onCLS(handleMetric);
  onFID(handleMetric);
  onLCP(handleMetric);

  // Other metrics
  onFCP(handleMetric);
  onTTFB(handleMetric);
  onINP(handleMetric);

  logger.info("Web Vitals monitoring initialized");
}

/**
 * Handle a web vitals metric
 */
function handleMetric(metric: Metric): void {
  const { name, value, rating, delta } = metric;

  // Record in metrics collector
  metricsCollector.observeHistogram(`webvitals_${name.toLowerCase()}`, value / 1000);
  metricsCollector.setGauge(`webvitals_${name.toLowerCase()}_rating`, ratingToScore(rating));

  // Log if rating is poor
  if (rating === "poor") {
    logger.warn(`Poor Web Vital: ${name}`, {
      component: "WebVitals",
      metric: name,
      value,
      rating,
      delta,
    });
  } else {
    logger.debug(`Web Vital: ${name}`, {
      component: "WebVitals",
      metric: name,
      value,
      rating,
      delta,
    });
  }
}

/**
 * Convert rating to numeric score
 */
function ratingToScore(rating: "good" | "needs-improvement" | "poor"): number {
  switch (rating) {
    case "good":
      return 1;
    case "needs-improvement":
      return 0.5;
    case "poor":
      return 0;
  }
}

/**
 * Get current Web Vitals metrics
 */
export function getWebVitalsMetrics(): Record<string, any> {
  const metrics = metricsCollector.getMetricsJSON();

  // Extract Web Vitals metrics
  const webVitals: Record<string, any> = {};
  for (const [key, value] of Object.entries(metrics.histograms)) {
    if (key.startsWith("webvitals_")) {
      webVitals[key.replace("webvitals_", "")] = value;
    }
  }

  return webVitals;
}
