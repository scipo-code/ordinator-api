import { Container } from "@/components/Container";
import { useParams } from "react-router-dom";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import ResourceOverview from "./resourceview/ResourceOverview";

export default function ResourceView() {
  const { asset } = useParams<{ asset: string}>();

  if (!asset) {
    throw new Error("Asset is required");
  }

  return (
    <Container maxWidth="full" padding="sm" className="bg-white flex-1">
      <Tabs defaultValue="resource-people" className="w-full">
        <TabsList>
          <TabsTrigger value="resource-people">People Resources</TabsTrigger>
          <TabsTrigger value="resource-aggregated">Aggregated Resources</TabsTrigger>
        </TabsList>
        <TabsContent value="resource-people">
          <p>Resource people table </p>
        </TabsContent>
        <TabsContent value="resource-aggregated">
          <ResourceOverview asset={asset}/>
        </TabsContent>
      </Tabs>
    </Container>
  );
}
