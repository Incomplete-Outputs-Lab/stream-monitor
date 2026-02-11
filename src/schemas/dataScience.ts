import { z } from 'zod';

// ========== Phase 1: Text Analysis ==========

/**
 * Word frequency schema
 */
export const WordFrequencySchema = z.object({
  word: z.string(),
  count: z.number(),
  percentage: z.number(),
});

/**
 * Word frequency result schema
 */
export const WordFrequencyResultSchema = z.object({
  words: z.array(WordFrequencySchema),
  totalWords: z.number(),
  uniqueWords: z.number(),
  avgWordsPerMessage: z.number(),
  totalMessages: z.number(),
});

/**
 * Emote usage schema
 */
export const EmoteUsageSchema = z.object({
  name: z.string(),
  count: z.number(),
  users: z.number(),
  percentage: z.number(),
});

/**
 * Hourly emote pattern schema
 */
export const HourlyEmotePatternSchema = z.object({
  hour: z.number(),
  count: z.number(),
});

/**
 * Emote analysis result schema
 */
export const EmoteAnalysisResultSchema = z.object({
  emotes: z.array(EmoteUsageSchema),
  totalEmoteUses: z.number(),
  emotePerMessageRate: z.number(),
  hourlyPattern: z.array(HourlyEmotePatternSchema),
});

/**
 * Length distribution schema
 */
export const LengthDistributionSchema = z.object({
  bucket: z.string(),
  count: z.number(),
  percentage: z.number(),
});

/**
 * Segment length stats schema
 */
export const SegmentLengthStatsSchema = z.object({
  segment: z.string(),
  avgLength: z.number(),
  messageCount: z.number(),
});

/**
 * Message length stats schema
 */
export const MessageLengthStatsSchema = z.object({
  avgLength: z.number(),
  medianLength: z.number(),
  stdDev: z.number(),
  minLength: z.number(),
  maxLength: z.number(),
  distribution: z.array(LengthDistributionSchema),
  bySegment: z.array(SegmentLengthStatsSchema),
});

// ========== Phase 2: Correlation Analysis ==========

/**
 * Scatter point schema
 */
export const ScatterPointSchema = z.object({
  viewers: z.number(),
  chats: z.number(),
  timestamp: z.string(),
});

/**
 * Hourly correlation schema
 */
export const HourlyCorrelationSchema = z.object({
  hour: z.number(),
  correlation: z.number(),
  sampleCount: z.number(),
});

/**
 * Correlation result schema
 */
export const CorrelationResultSchema = z.object({
  pearsonCoefficient: z.number(),
  interpretation: z.string(),
  scatterData: z.array(ScatterPointSchema),
  hourlyCorrelation: z.array(HourlyCorrelationSchema),
});

/**
 * Category change schema (correlation)
 */
export const CategoryChangeCorrelationSchema = z.object({
  timestamp: z.string(),
  fromCategory: z.string(),
  toCategory: z.string(),
  beforeViewers: z.number(),
  afterViewers: z.number(),
  viewerChangePercent: z.number(),
  chatChangePercent: z.number(),
});

/**
 * Category performance schema
 */
export const CategoryPerformanceSchema = z.object({
  category: z.string(),
  avgViewers: z.number(),
  avgChatRate: z.number(),
  totalTimeMinutes: z.number(),
  changeCount: z.number(),
});

/**
 * Category impact result schema
 */
export const CategoryImpactResultSchema = z.object({
  changes: z.array(CategoryChangeCorrelationSchema),
  categoryPerformance: z.array(CategoryPerformanceSchema),
});

// ========== Phase 3: User Behavior Analysis ==========

/**
 * Chatter activity score schema
 */
export const ChatterActivityScoreSchema = z.object({
  userName: z.string(),
  score: z.number(),
  messageCount: z.number(),
  streamCount: z.number(),
  badges: z.array(z.string()),
  rank: z.number(),
});

/**
 * Score distribution schema
 */
export const ScoreDistributionSchema = z.object({
  scoreRange: z.string(),
  userCount: z.number(),
  percentage: z.number(),
});

/**
 * Segment avg score schema
 */
export const SegmentAvgScoreSchema = z.object({
  segment: z.string(),
  avgScore: z.number(),
  userCount: z.number(),
});

/**
 * Chatter score result schema
 */
export const ChatterScoreResultSchema = z.object({
  scores: z.array(ChatterActivityScoreSchema),
  scoreDistribution: z.array(ScoreDistributionSchema),
  segmentAvgScores: z.array(SegmentAvgScoreSchema),
});

// ========== Phase 4: Anomaly Detection ==========

/**
 * Anomaly schema
 */
export const AnomalySchema = z.object({
  timestamp: z.string(),
  value: z.number(),
  previousValue: z.number(),
  changeAmount: z.number(),
  changeRate: z.number(),
  modifiedZScore: z.number(),
  isPositive: z.boolean(),
  minutesFromStreamStart: z.number().optional(),
  streamPhase: z.string(),
  streamId: z.number().optional(),
});

/**
 * Trend stats schema
 */
export const TrendStatsSchema = z.object({
  viewerTrend: z.string(),
  viewerMedian: z.number(),
  viewerMad: z.number(),
  viewerAvg: z.number(),
  viewerStdDev: z.number(),
  chatTrend: z.string(),
  chatMedian: z.number(),
  chatMad: z.number(),
  chatAvg: z.number(),
  chatStdDev: z.number(),
});

/**
 * Anomaly result schema
 */
export const AnomalyResultSchema = z.object({
  viewerAnomalies: z.array(AnomalySchema),
  chatAnomalies: z.array(AnomalySchema),
  trendStats: TrendStatsSchema,
});

/**
 * Data science query schema
 */
export const DataScienceQuerySchema = z.object({
  channelId: z.number().optional(),
  streamId: z.number().optional(),
  startTime: z.string().optional(),
  endTime: z.string().optional(),
  limit: z.number().optional(),
  zThreshold: z.number().optional(),
});

// Export types
export type WordFrequency = z.infer<typeof WordFrequencySchema>;
export type WordFrequencyResult = z.infer<typeof WordFrequencyResultSchema>;
export type EmoteUsage = z.infer<typeof EmoteUsageSchema>;
export type HourlyEmotePattern = z.infer<typeof HourlyEmotePatternSchema>;
export type EmoteAnalysisResult = z.infer<typeof EmoteAnalysisResultSchema>;
export type LengthDistribution = z.infer<typeof LengthDistributionSchema>;
export type SegmentLengthStats = z.infer<typeof SegmentLengthStatsSchema>;
export type MessageLengthStats = z.infer<typeof MessageLengthStatsSchema>;
export type ScatterPoint = z.infer<typeof ScatterPointSchema>;
export type HourlyCorrelation = z.infer<typeof HourlyCorrelationSchema>;
export type CorrelationResult = z.infer<typeof CorrelationResultSchema>;
export type CategoryChangeCorrelation = z.infer<typeof CategoryChangeCorrelationSchema>;
export type CategoryPerformance = z.infer<typeof CategoryPerformanceSchema>;
export type CategoryImpactResult = z.infer<typeof CategoryImpactResultSchema>;
export type ChatterActivityScore = z.infer<typeof ChatterActivityScoreSchema>;
export type ScoreDistribution = z.infer<typeof ScoreDistributionSchema>;
export type SegmentAvgScore = z.infer<typeof SegmentAvgScoreSchema>;
export type ChatterScoreResult = z.infer<typeof ChatterScoreResultSchema>;
export type Anomaly = z.infer<typeof AnomalySchema>;
export type TrendStats = z.infer<typeof TrendStatsSchema>;
export type AnomalyResult = z.infer<typeof AnomalyResultSchema>;
export type DataScienceQuery = z.infer<typeof DataScienceQuerySchema>;
