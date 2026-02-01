import { PieChart as RechartsPieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";

interface ChartDataPoint {
  [key: string]: string | number | undefined;
}

interface PieChartProps {
  data: ChartDataPoint[];
  dataKey: string;
  nameKey?: string;
  colors?: string[];
  title?: string;
  showPercentage?: boolean;
  innerRadius?: number;
  outerRadius?: number;
  height?: number;
  showLegend?: boolean;
}

const DEFAULT_COLORS = [
  '#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6',
  '#ec4899', '#14b8a6', '#f97316', '#06b6d4', '#84cc16',
];

export function PieChart({
  data,
  dataKey,
  nameKey = "name",
  colors = DEFAULT_COLORS,
  title,
  showPercentage = true,
  innerRadius = 0,
  outerRadius = 80,
  height = 300,
  showLegend = true,
}: PieChartProps) {
  // Calculate total for percentage
  const total = data.reduce((sum, entry) => {
    const value = entry[dataKey];
    return sum + (typeof value === 'number' ? value : 0);
  }, 0);

  const renderLabel = (entry: any) => {
    if (!showPercentage) return '';
    const value = entry[dataKey];
    const percent = typeof value === 'number' ? ((value / total) * 100).toFixed(1) : '0';
    return `${percent}%`;
  };

  return (
    <div className="w-full">
      {title && (
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{title}</h3>
      )}
      <div style={{ height: `${height}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <RechartsPieChart>
            <Pie
              data={data}
              dataKey={dataKey}
              nameKey={nameKey}
              cx="50%"
              cy="50%"
              innerRadius={innerRadius}
              outerRadius={outerRadius}
              label={renderLabel}
              labelLine={false}
            >
              {data.map((_entry, index) => (
                <Cell key={`cell-${index}`} fill={colors[index % colors.length]} />
              ))}
            </Pie>
            <Tooltip
              contentStyle={{
                backgroundColor: '#ffffff',
                border: '1px solid #e0e0e0',
                borderRadius: '6px',
                fontSize: '12px'
              }}
              formatter={(value: any) => {
                if (typeof value === 'number') {
                  const percent = ((value / total) * 100).toFixed(1);
                  return `${value.toLocaleString()} (${percent}%)`;
                }
                return value;
              }}
            />
            {showLegend && (
              <Legend
                verticalAlign="middle"
                align="right"
                layout="vertical"
                iconType="circle"
              />
            )}
          </RechartsPieChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
