import { BarChart as RechartsBarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from "recharts";

interface ChartDataPoint {
  [key: string]: string | number | undefined;
}

interface HorizontalBarChartProps {
  data: ChartDataPoint[];
  dataKey: string;
  nameKey?: string;
  color?: string;
  title?: string;
  maxItems?: number;
  height?: number;
  showLegend?: boolean;
  xAxisLabel?: string;
}

export function HorizontalBarChart({
  data,
  dataKey,
  nameKey = "name",
  color = "#10b981",
  title,
  maxItems,
  height = 400,
  showLegend = false,
  xAxisLabel,
}: HorizontalBarChartProps) {
  // Limit data if maxItems is specified
  const displayData = maxItems ? data.slice(0, maxItems) : data;
  
  // Calculate dynamic height based on number of items
  const calculatedHeight = Math.max(height, displayData.length * 40);

  return (
    <div className="w-full">
      {title && (
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{title}</h3>
      )}
      <div style={{ height: `${calculatedHeight}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <RechartsBarChart
            data={displayData}
            layout="vertical"
            margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
          >
            <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
            <XAxis
              type="number"
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              label={xAxisLabel ? { value: xAxisLabel, position: 'insideBottom', offset: -10 } : undefined}
            />
            <YAxis
              type="category"
              dataKey={nameKey}
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              width={150}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: '#ffffff',
                border: '1px solid #e0e0e0',
                borderRadius: '6px',
                fontSize: '12px'
              }}
              formatter={(value: any) => {
                if (typeof value === 'number') {
                  return value.toLocaleString();
                }
                return value;
              }}
            />
            {showLegend && <Legend />}
            <Bar
              dataKey={dataKey}
              fill={color}
              radius={[0, 4, 4, 0]}
            />
          </RechartsBarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
