import { Container } from "@/components/Container";
import { useParams } from "react-router-dom";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import ResourceOverview from "./dashboard/ResourceOverview";
import ExportDialog from "@/components/assetpage/ExportDialog";
import { ResourceChart } from "./dashboard/ResouceChart";

export default function AssetDashboard() {
  const { asset } = useParams<{ asset: string}>();

  if (!asset) {
    throw new Error("Asset is required");
  }

  return (
    <Container maxWidth="full" padding="sm" className="bg-white min-h-screen">
      <h1 className="text-4xl font-bold">Dashboard: {asset.toUpperCase()}</h1>
      <br/>
      <Tabs defaultValue="resource-loading-graph" className="w-full">
        <div className="flex items-center justify-between">
          <TabsList>
            <TabsTrigger value="resource-loading-graph">Graph</TabsTrigger>
            <TabsTrigger value="resource-overview">Resources</TabsTrigger>
          </TabsList>
          <ExportDialog asset={asset}/>
        </div>
          <TabsContent value="resource-loading-graph">
            <ResourceChart asset={asset}/>
          </TabsContent>
          <TabsContent value="resource-overview">
            <ResourceOverview asset={asset}/>
          </TabsContent>
        </Tabs>
    </Container>
  );
}
