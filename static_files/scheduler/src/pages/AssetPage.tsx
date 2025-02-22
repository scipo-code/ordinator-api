import { Container } from "@/components/Container";
import { useParams } from "react-router-dom";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import ResourceOverview from "./dashboard/ResourceOverview";
import ExportDialog from "@/components/assetpage/ExportDialog";

export default function AssetPage() {
  const { asset } = useParams<{ asset: string}>();

  if (!asset) {
    throw new Error("Asset is required");
  }

  return (
    <Container maxWidth="full" padding="sm" className="bg-white min-h-screen">
      <h1 className="text-4xl font-bold">Dashboard</h1>
      <br/>
      <div className="flex items-center justify-between">
        <div className="flex-1">
        <Tabs defaultValue="loading-graph" className="max-w-screen-lg">
          <TabsList>
            <TabsTrigger value="loading-graph">Graph</TabsTrigger>
            <TabsTrigger value="loading-page">Loadings</TabsTrigger>
          </TabsList>
          <TabsContent value="loading-graph">
            <p>Graph</p>
          </TabsContent>
          <TabsContent value="loading-page">
            <ResourceOverview />
          </TabsContent>
        </Tabs>
      </div>
      <div className="ml-4">
        <ExportDialog asset={asset}/>
      </div>
    </div>
      
    </Container>
  );
}
