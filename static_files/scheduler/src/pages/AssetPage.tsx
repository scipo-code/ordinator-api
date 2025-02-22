import { Container } from "@/components/Container";
// import { useParams } from "react-router-dom";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import ResourceOverview from "./dashboard/ResourceOverview";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

export default function AssetPage() {
  // TODO: Use the url to populate
  // const { asset } = useParams<{ asset: string}>();

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
        <Dialog>
          <DialogTrigger asChild>
            <Button variant="default">Export</Button>
            </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Export current workorders to Excel</DialogTitle>
              <DialogDescription>
                This will export the current workorders to excel. Working excel disconnects changes from the scheduling system.
              </DialogDescription>
            </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="filename" className="text-right">
                Filename
              </Label>
              <Input id="filename" value="workorder_schedules.xlsx" className="col-span-3" />
            </div>
          </div>
          <DialogFooter>
            <Button type="submit">Download</Button>
          </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    </div>
      
    </Container>
  );
}
