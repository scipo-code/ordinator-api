import { Container } from "@/components/Container";
import { type ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart";
import { getResources } from "@/hooks/GetResources";
import { Asset } from "@/types";
import { ReloadIcon } from "@radix-ui/react-icons";
import { Bar, BarChart, CartesianGrid, XAxis } from "recharts";


const fallbackColors = [
  "#2563eb",
  "#60a5fa",
  "#facc15",
  "#22c55e",
  "#ef4444",
  "#ec4899",
  "#8b5cf6",
];


export function ResourceChart({ asset }: Asset) {
  const {
    data,
    error,
    isLoading,
  } = getResources(asset);

  if (isLoading) {
    return <div className="p-4 flex item-center gap-2">
      <ReloadIcon className="animate-spin"/>Loading...
    </div>
  }

  if (error) {
    return <div className="p-4 text-red-600">
      Error loading resources: {(error as Error).message}
    </div>
  }

  console.log(data);

  if (!data || data.data.length === 0) {
    return <div className="p-4">No resources found</div>
  }
  
  const chartData = data.data.map((item) => ({
    periodId: item.periodId,
    ...item.values,
  }));

  const chartConfig: ChartConfig = data.metadata.resources.reduce(
    (acc, r, index) => {
      const color = fallbackColors[index % fallbackColors.length];
      acc[r.id] = {
        label: r.label,
        color: color,
      };
      return acc
    },
    {} as ChartConfig
  );

  return (

    <Container maxWidth="full" padding="sm" className="bg-white border border-gray-300 shadow rounded-lg">
      <ChartContainer config={chartConfig} className="min-h-[200px] max-w-[600px]">
        <BarChart accessibilityLayer data={chartData}>
          <CartesianGrid vertical={false}/>
          <ChartTooltip content={<ChartTooltipContent/>} />
          {data.metadata.resources.map((resource) => (
            <Bar
              key={resource.id}
              dataKey={resource.id}
              fill={`var(--color-${resource.id})`}
              radius={4}
              stackId="a"
              isAnimationActive={false}
            />
        
          ))}
          <XAxis
            dataKey="periodId"
            tickLine={false}
            tickMargin={10}
            axisLine={false}
          />
        </BarChart>
      </ChartContainer>
    </Container>
  )
}
