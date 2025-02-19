import { Container } from "@/components/Container";
// import { useParams } from "react-router-dom";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import ResourceOverview from "./dashboard/ResourceOverview";

export default function AssetPage() {
  // TODO: Use the url to populate
  // const { asset } = useParams<{ asset: string}>();

  return (
    <Container maxWidth="full" padding="sm" className="bg-white min-h-screen">
      <h1 className="text-4xl font-bold">Dashboard</h1>
      <br/>
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
          
      
    </Container>
  );
}
