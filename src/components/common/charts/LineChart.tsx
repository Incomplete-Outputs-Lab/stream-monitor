import { LineChart as RechartsLineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from "recharts";

interface ChartDataPoint {
  [key: string]: string | number | undefined;
}

interface LineChartProps {
  data: ChartDataPoint[];
  dataKey: string;
  xAxisKey?: string;
  color?: string;
  title?: string;
  height?: number;
  showLegend?: boolean;
  yAxisLabel?: string;
}

export function LineChart({
  data,
  dataKey,
  xAxisKey = "time",
  color = "#3b82f6",
  title,
  height = 300,
  showLegend = false,
  yAxisLabel,
}: LineChartProps) {
  return (
    <div className="w-full">
      {title && (
        <h3 className="text-lg font-semibold text-gray-900 mb-4">{title}</h3>
      )}
      <div style={{ height: `${height}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <RechartsLineChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
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
            <Line
              type="monotone"
              dataKey={dataKey}
              stroke={color}
              strokeWidth={2}
              dot={{ fill: color, strokeWidth: 2, r: 4 }}
              activeDot={{ r: 6, stroke: color, strokeWidth: 2, fill: '#ffffff' }}
            />
          </RechartsLineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}