import { Container } from "@/components/Container";
import { useParams } from "react-router-dom";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import ResourceOverview from "./dashboard/ResourceOverview";
import ExportDialog from "@/components/assetpage/ExportDialog";

export default function AssetDashboard() {
  const { asset } = useParams<{ asset: string}>();

  if (!asset) {
    throw new Error("Asset is required");
  }

  return (
    <Container maxWidth="full" padding="sm" className="bg-white min-h-screen">
      <h1 className="text-4xl font-bold">Dashboard</h1>
      <br/>
      <Tabs defaultValue="loading-graph" className="w-full">
        <div className="flex items-center justify-between">
          <TabsList>
            <TabsTrigger value="loading-graph">Graph</TabsTrigger>
            <TabsTrigger value="loading-page">Loadings</TabsTrigger>
          </TabsList>
          <ExportDialog asset={asset}/>
        </div>
          <TabsContent value="loading-graph">
            <p>Graph</p>
          </TabsContent>
          <TabsContent value="loading-page">
            <ResourceOverview />
          </TabsContent>
        </Tabs>
    </Container>
  );
}
