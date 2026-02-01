import { useCallback } from 'react';

/**
 * 共通のフォーマット関数を提供するフック
 */
export function useFormatting() {
  const formatNumber = useCallback((num: number) => {
    return new Intl.NumberFormat('ja-JP').format(num);
  }, []);

  const formatHours = useCallback((minutes: number) => {
    return (minutes / 60).toFixed(1);
  }, []);

  const formatDate = useCallback((date: Date | string) => {
    const dateObj = typeof date === 'string' ? new Date(date) : date;
    return new Intl.DateTimeFormat('ja-JP').format(dateObj);
  }, []);

  return { formatNumber, formatHours, formatDate };
}
