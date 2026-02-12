interface SkeletonProps {
  className?: string;
  variant?: 'text' | 'rectangular' | 'circular';
  width?: string | number;
  height?: string | number;
  count?: number;
}

export const Skeleton = ({
  className = '',
  variant = 'rectangular',
  width,
  height,
  count = 1,
}: SkeletonProps) => {
  const getVariantClasses = () => {
    switch (variant) {
      case 'text':
        return 'h-4 rounded';
      case 'circular':
        return 'rounded-full';
      case 'rectangular':
      default:
        return 'rounded-lg';
    }
  };

  const style = {
    width: width ? (typeof width === 'number' ? `${width}px` : width) : undefined,
    height: height ? (typeof height === 'number' ? `${height}px` : height) : undefined,
  };

  const skeletonElement = (
    <div
      className={`animate-pulse bg-gray-200 dark:bg-gray-700 ${getVariantClasses()} ${className}`}
      style={style}
    />
  );

  if (count > 1) {
    return (
      <div className="space-y-3">
        {Array.from({ length: count }).map((_, index) => (
          <div key={index}>{skeletonElement}</div>
        ))}
      </div>
    );
  }

  return skeletonElement;
};

// 統計カード用Skeleton
export const StatCardSkeleton = () => (
  <div className="bg-white dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
    <Skeleton variant="text" width="60%" className="mb-2" />
    <Skeleton variant="text" width="40%" height={32} />
  </div>
);

// テーブル用Skeleton
export const TableSkeleton = ({ rows = 5, columns = 5 }: { rows?: number; columns?: number }) => (
  <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
    <div className="p-4 border-b border-gray-200 dark:border-gray-700">
      <Skeleton variant="text" width="30%" height={24} />
    </div>
    <div className="p-4 space-y-3">
      {/* Header */}
      <div className="flex gap-4">
        {Array.from({ length: columns }).map((_, i) => (
          <Skeleton key={i} variant="text" width="100%" height={20} />
        ))}
      </div>
      {/* Rows */}
      {Array.from({ length: rows }).map((_, rowIndex) => (
        <div key={rowIndex} className="flex gap-4">
          {Array.from({ length: columns }).map((_, colIndex) => (
            <Skeleton key={colIndex} variant="text" width="100%" height={16} />
          ))}
        </div>
      ))}
    </div>
  </div>
);

// チャート用Skeleton
export const ChartSkeleton = ({ height = 300 }: { height?: number }) => (
  <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
    <Skeleton variant="text" width="40%" height={24} className="mb-4" />
    <Skeleton variant="rectangular" width="100%" height={height} />
  </div>
);

// チャットメッセージリスト用Skeleton
export const ChatMessagesSkeleton = ({ count = 8 }: { count?: number }) => (
  <div className="space-y-3">
    {Array.from({ length: count }).map((_, index) => (
      <div key={index} className="flex flex-col space-y-1 p-2">
        {/* Time and User */}
        <div className="flex items-center gap-2">
          <Skeleton variant="text" width={80} height={12} />
          <Skeleton variant="text" width={120} height={12} />
          <Skeleton variant="rectangular" width={40} height={16} className="rounded" />
        </div>
        {/* Message */}
        <Skeleton variant="text" width="90%" height={14} />
      </div>
    ))}
  </div>
);

// 統計ダッシュボード用Skeleton（サマリーカード + チャート）
export const StatsDashboardSkeleton = ({
  cardCount = 3,
  chartCount = 1
}: {
  cardCount?: number;
  chartCount?: number;
}) => {
  const getGridClass = () => {
    switch (cardCount) {
      case 1:
        return 'grid grid-cols-1 gap-4';
      case 2:
        return 'grid grid-cols-1 md:grid-cols-2 gap-4';
      case 3:
        return 'grid grid-cols-1 md:grid-cols-3 gap-4';
      case 4:
        return 'grid grid-cols-1 md:grid-cols-4 gap-4';
      default:
        return 'grid grid-cols-1 md:grid-cols-3 gap-4';
    }
  };

  return (
    <div className="space-y-6">
      {/* Summary Cards */}
      <div className={getGridClass()}>
        {Array.from({ length: cardCount }).map((_, index) => (
          <StatCardSkeleton key={index} />
        ))}
      </div>
      {/* Charts */}
      {Array.from({ length: chartCount }).map((_, index) => (
        <ChartSkeleton key={index} />
      ))}
    </div>
  );
};

// フォーム用Skeleton
export const FormSkeleton = ({ rows = 4 }: { rows?: number }) => (
  <div className="bg-white dark:bg-gray-800 p-6 rounded-lg border border-gray-200 dark:border-gray-700 space-y-4">
    {Array.from({ length: rows }).map((_, index) => (
      <div key={index} className="space-y-2">
        <Skeleton variant="text" width="30%" height={16} />
        <Skeleton variant="rectangular" width="100%" height={40} className="rounded" />
      </div>
    ))}
  </div>
);
