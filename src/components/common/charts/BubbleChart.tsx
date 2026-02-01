import { ScatterChart, Scatter, XAxis, YAxis, ZAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from "recharts";

interface BubbleDataPoint {
  [key: string]: string | number | undefined;
}

interface BubbleChartProps {
  data: BubbleDataPoint[];
  xDataKey: string;
  yDataKey: string;
  zDataKey: string;
  nameKey?: string;
  xAxisLabel?: string;
  yAxisLabel?: string;
  color?: string;
  title?: string;
  height?: number;
  showLegend?: boolean;
}

export function BubbleChart({
  data,
  xDataKey,
  yDataKey,
  zDataKey,
  nameKey = "name",
  xAxisLabel,
  yAxisLabel,
  color = "#3b82f6",
  title,
  height = 400,
  showLegend = false,
}: BubbleChartProps) {
  // Find min/max for Z-axis scaling
  const zValues = data
    .map(item => item[zDataKey])
    .filter((val): val is number => typeof val === 'number');
  
  const minZ = Math.min(...zValues);
  const maxZ = Math.max(...zValues);

  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload;
      return (
        <div className="bg-white border border-gray-200 rounded-md p-2 text-xs shadow-lg">
          <p className="font-semibold">{data[nameKey]}</p>
          <p className="text-gray-600">{xAxisLabel || xDataKey}: {data[xDataKey]}</p>
          <p className="text-gray-600">{yAxisLabel || yDataKey}: {data[yDataKey]}</p>
          <p className="text-gray-600">{zDataKey}: {typeof data[zDataKey] === 'number' ? data[zDataKey].toLocaleString() : data[zDataKey]}</p>
        </div>
      );
    }
    return null;
  };

  return (
    <div className="w-full">
      {title && (
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{title}</h3>
      )}
      <div style={{ height: `${height}px` }}>
        <ResponsiveContainer width="100%" height="100%">
          <ScatterChart margin={{ top: 20, right: 20, bottom: 20, left: 20 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="#f0f0f0" />
            <XAxis
              type="number"
              dataKey={xDataKey}
              name={xAxisLabel || xDataKey}
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              label={xAxisLabel ? { value: xAxisLabel, position: 'insideBottom', offset: -10 } : undefined}
            />
            <YAxis
              type="number"
              dataKey={yDataKey}
              name={yAxisLabel || yDataKey}
              tick={{ fontSize: 12 }}
              tickLine={{ stroke: '#e0e0e0' }}
              label={yAxisLabel ? { value: yAxisLabel, angle: -90, position: 'insideLeft' } : undefined}
            />
            <ZAxis
              type="number"
              dataKey={zDataKey}
              range={[100, 1000]}
              name={zDataKey}
            />
            <Tooltip content={<CustomTooltip />} />
            {showLegend && <Legend />}
            <Scatter
              name={title || "Data"}
              data={data}
              fill={color}
              fillOpacity={0.6}
            />
          </ScatterChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
