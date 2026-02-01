import { SortDirection } from '../../hooks/useSortableData';

interface SortableTableHeaderProps {
  /** 表示テキスト */
  children: React.ReactNode;
  /** ソート対象のキー */
  sortKey: string;
  /** 現在のソート設定のキー */
  currentSortKey: string;
  /** 現在のソート方向 */
  currentDirection: SortDirection;
  /** ソートリクエストハンドラー */
  onSort: (key: string) => void;
  /** テキストの配置（left または right） */
  align?: 'left' | 'right';
  /** 追加のクラス名 */
  className?: string;
}

/**
 * ソート可能なテーブルヘッダーコンポーネント
 */
export function SortableTableHeader({
  children,
  sortKey,
  currentSortKey,
  currentDirection,
  onSort,
  align = 'left',
  className = '',
}: SortableTableHeaderProps) {
  const isActive = currentSortKey === sortKey;
  const textAlign = align === 'right' ? 'text-right' : 'text-left';

  return (
    <th
      className={`px-4 py-3 ${textAlign} text-xs font-medium text-gray-400 uppercase tracking-wider cursor-pointer hover:bg-gray-800/50 transition-colors select-none ${className}`}
      onClick={() => onSort(sortKey)}
    >
      <div className={`flex items-center gap-1 ${align === 'right' ? 'justify-end' : ''}`}>
        <span>{children}</span>
        <span className="inline-flex flex-col text-[10px] leading-none">
          {isActive && currentDirection === 'asc' && (
            <span className="text-blue-400">▲</span>
          )}
          {isActive && currentDirection === 'desc' && (
            <span className="text-blue-400">▼</span>
          )}
          {!isActive && (
            <span className="text-gray-600 opacity-50">▲</span>
          )}
        </span>
      </div>
    </th>
  );
}
