import { Container } from "@/components/Container";
import { useParams } from "react-router-dom";
import PlanningTable from "./planningview/PlanningTable"; 

export default function PlanningView() {
  const { asset } = useParams<{ asset: string}>();
  console.log("Here")

  if (!asset) {
    throw new Error("Asset is required");
  }

  return (
    <Container maxWidth="full" padding="sm" className="bg-white flex-1">
      <div className="w-[400px] flex gap-2 ">
        <p>Back</p>
        <p>Week</p>
        <p>Next</p>
      </div>
      <br />
      <PlanningTable asset={asset}/>
    </Container>
  );
}
