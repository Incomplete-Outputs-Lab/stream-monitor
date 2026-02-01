import { ComposedChart, Bar, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from "recharts";

interface ChartDataPoint {
  [key: string]: string | number | undefined;
}

interface DualAxisChartProps {
  data: ChartDataPoint[];
  primaryDataKey: string;
  secondaryDataKey: string;
  primaryType?: 'bar' | 'line';
  secondaryType?: 'bar' | 'line';
  xAxisKey?: string;
  primaryColor?: string;
  secondaryColor?: string;
  primaryYAxisLabel?: string;
  secondaryYAxisLabel?: string;
  title?: string;
  height?: number;
  showLegend?: boolean;
}

export function DualAxisChart({
  data,
  primaryDataKey,
  secondaryDataKey,
  primaryType = 'bar',
  secondaryType = 'line',
  xAxisKey = "date",
  primaryColor = "#3b82f6",
  secondaryColor = "#10b981",
  primaryYAxisLabel,
  secondaryYAxisLabel,
  title,
  height = 300,
  showLegend = true,
}: DualAxisChartProps) {
  return (
    <div className="w-full">
      {title && (
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{title}</h3>
      )}
      <div style={{ height: `${height}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <ComposedChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
            <XAxis
              dataKey={xAxisKey}
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
            />
            <YAxis
              yAxisId="left"
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              label={primaryYAxisLabel ? { value: primaryYAxisLabel, angle: -90, position: 'insideLeft' } : undefined}
            />
            <YAxis
              yAxisId="right"
              orientation="right"
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              label={secondaryYAxisLabel ? { value: secondaryYAxisLabel, angle: 90, position: 'insideRight' } : undefined}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: '#ffffff',
                border: '1px solid #e0e0e0',
                borderRadius: '6px',
                fontSize: '12px'
              }}
            />
            {showLegend && <Legend />}
            
            {primaryType === 'bar' ? (
              <Bar
                yAxisId="left"
                dataKey={primaryDataKey}
                fill={primaryColor}
                radius={[4, 4, 0, 0]}
              />
            ) : (
              <Line
                yAxisId="left"
                type="monotone"
                dataKey={primaryDataKey}
                stroke={primaryColor}
                strokeWidth={2}
                dot={{ fill: primaryColor, strokeWidth: 2, r: 4 }}
              />
            )}
            
            {secondaryType === 'bar' ? (
              <Bar
                yAxisId="right"
                dataKey={secondaryDataKey}
                fill={secondaryColor}
                radius={[4, 4, 0, 0]}
              />
            ) : (
              <Line
                yAxisId="right"
                type="monotone"
                dataKey={secondaryDataKey}
                stroke={secondaryColor}
                strokeWidth={2}
                dot={{ fill: secondaryColor, strokeWidth: 2, r: 4 }}
              />
            )}
          </ComposedChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
