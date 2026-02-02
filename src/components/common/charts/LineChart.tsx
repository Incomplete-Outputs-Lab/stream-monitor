import { LineChart as RechartsLineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from "recharts";

interface ChartDataPoint {
  [key: string]: string | number | undefined;
}

interface LineConfig {
  key: string;
  color?: string;
  yAxisId?: 'left' | 'right';
}

interface LineChartProps {
  data: ChartDataPoint[];
  // 新規: 複数ライン用
  lines?: LineConfig[];
  xKey?: string;  // xAxisKeyのエイリアス
  // 既存互換
  dataKey?: string;
  xAxisKey?: string;
  color?: string;
  title?: string;
  height?: number;
  showLegend?: boolean;
  yAxisLabel?: string;
}

export function LineChart({
  data,
  lines,
  xKey,
  dataKey,
  xAxisKey,
  color = "#3b82f6",
  title,
  height = 300,
  showLegend = false,
  yAxisLabel,
}: LineChartProps) {
  // xKeyとxAxisKeyの統一（xKeyを優先）
  const finalXAxisKey = xKey || xAxisKey || "time";
  
  // 複数ライン or 単一ライン
  const lineConfigs: LineConfig[] = lines || (dataKey ? [{ key: dataKey, color, yAxisId: 'left' }] : []);
  
  // 右軸が必要かどうかを判定
  const hasRightAxis = lineConfigs.some(line => line.yAxisId === 'right');
  
  return (
    <div className="w-full">
      {title && (
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">{title}</h3>
      )}
      <div style={{ height: `${height}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <RechartsLineChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
            <XAxis
              dataKey={finalXAxisKey}
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
            />
            <YAxis
              yAxisId="left"
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              label={yAxisLabel ? { value: yAxisLabel, angle: -90, position: 'insideLeft' } : undefined}
            />
            {hasRightAxis && (
              <YAxis
                yAxisId="right"
                orientation="right"
                tick={{ fontSize: 12 }}
                tickLine={{ stroke: '#e0e0e0' }}
              />
            )}
            <Tooltip
              contentStyle={{
                backgroundColor: '#ffffff',
                border: '1px solid #e0e0e0',
                borderRadius: '6px',
                fontSize: '12px'
              }}
            />
            {showLegend && <Legend />}
            {lineConfigs.map((lineConfig, index) => (
              <Line
                key={lineConfig.key}
                yAxisId={lineConfig.yAxisId || 'left'}
                type="monotone"
                dataKey={lineConfig.key}
                stroke={lineConfig.color || `hsl(${(index * 60) % 360}, 70%, 50%)`}
                strokeWidth={2}
                dot={{ fill: lineConfig.color || `hsl(${(index * 60) % 360}, 70%, 50%)`, strokeWidth: 2, r: 4 }}
                activeDot={{ r: 6, strokeWidth: 2, fill: '#ffffff' }}
              />
            ))}
          </RechartsLineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}