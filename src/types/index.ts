/**
 * Type definitions - Re-exported from Zod schemas
 * All types are now strictly validated at runtime using Zod
 */

// Re-export all types from schemas
export * from '../schemas';

// Additional utility types
export interface ChartDataPoint {
  [key: string]: string | number | undefined;
}
