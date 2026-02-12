import { useMemo, useState } from 'react';

export type SortDirection = 'asc' | 'desc' | null;

export interface SortConfig<T> {
  key: keyof T | string;
  direction: SortDirection;
}

/**
 * テーブルデータのソート機能を提供するカスタムフック
 * @param items ソート対象のデータ配列
 * @param defaultSort デフォルトのソート設定（省略可）
 * @returns ソート済みデータ、現在のソート設定、ソートリクエスト関数
 */
export function useSortableData<T>(
  items: T[],
  defaultSort?: SortConfig<T>
) {
  const [sortConfig, setSortConfig] = useState<SortConfig<T>>(
    defaultSort || { key: '', direction: null }
  );

  const sortedItems = useMemo(() => {
    if (!sortConfig.direction || !sortConfig.key) {
      return items;
    }

    const sortableItems = [...items];
    sortableItems.sort((a, b) => {
      const key = sortConfig.key;
      
      // ネストされたキー（例: 'peak_ccu / average_ccu'）は特殊処理
      let aValue: any;
      let bValue: any;

      if (typeof key === 'string' && key.includes('/')) {
        // 計算値の場合（P/A Ratioなど）
        const keys = key.split('/').map(k => k.trim() as keyof T);
        const aNumerator = a[keys[0]] as number;
        const aDenominator = a[keys[1]] as number;
        const bNumerator = b[keys[0]] as number;
        const bDenominator = b[keys[1]] as number;
        
        aValue = aDenominator > 0 ? aNumerator / aDenominator : 0;
        bValue = bDenominator > 0 ? bNumerator / bDenominator : 0;
      } else {
        aValue = a[key as keyof T];
        bValue = b[key as keyof T];
      }

      // null/undefined を最後にソート
      if (aValue == null && bValue == null) return 0;
      if (aValue == null) return 1;
      if (bValue == null) return -1;

      // 文字列を数値に変換して比較を試みる（BIGINT対応）
      const aNum = typeof aValue === 'string' ? parseFloat(aValue) : aValue;
      const bNum = typeof bValue === 'string' ? parseFloat(bValue) : bValue;

      // 数値として比較可能な場合
      if (typeof aNum === 'number' && !isNaN(aNum) &&
          typeof bNum === 'number' && !isNaN(bNum)) {
        return sortConfig.direction === 'asc'
          ? aNum - bNum
          : bNum - aNum;
      }

      // 文字列比較（大文字小文字を区別しない）
      const aString = String(aValue).toLowerCase();
      const bString = String(bValue).toLowerCase();

      if (aString < bString) {
        return sortConfig.direction === 'asc' ? -1 : 1;
      }
      if (aString > bString) {
        return sortConfig.direction === 'asc' ? 1 : -1;
      }
      return 0;
    });

    return sortableItems;
  }, [items, sortConfig]);

  /**
   * ソート要求を処理
   * @param key ソート対象のキー
   */
  const requestSort = (key: keyof T | string) => {
    let direction: SortDirection = 'asc';
    
    if (sortConfig.key === key) {
      // 同じキーをクリック: asc -> desc -> null の順で切り替え
      if (sortConfig.direction === 'asc') {
        direction = 'desc';
      } else if (sortConfig.direction === 'desc') {
        direction = null;
      }
    }

    setSortConfig({ key, direction });
  };

  return { sortedItems, sortConfig, requestSort };
}
