import { BarChart as RechartsBarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from "recharts";

interface ChartDataPoint {
  [key: string]: string | number | undefined;
}

interface BarChartProps {
  data: ChartDataPoint[];
  dataKey: string;
  xAxisKey?: string;
  color?: string;
  title?: string;
  height?: number;
  showLegend?: boolean;
  yAxisLabel?: string;
}

export function BarChart({
  data,
  dataKey,
  xAxisKey = "name",
  color = "#10b981",
  title,
  height = 300,
  showLegend = false,
  yAxisLabel,
}: BarChartProps) {
  return (
    <div className="w-full">
      {title && (
        <h3 className="text-lg font-semibold text-gray-900 mb-4">{title}</h3>
      )}
      <div style={{ height: `${height}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <RechartsBarChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
            <XAxis
              dataKey={xAxisKey}
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
            />
            <YAxis
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              label={yAxisLabel ? { value: yAxisLabel, angle: -90, position: 'insideLeft' } : undefined}
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
            <Bar
              dataKey={dataKey}
              fill={color}
              radius={[4, 4, 0, 0]}
            />
          </RechartsBarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}