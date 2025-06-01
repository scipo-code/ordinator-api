import { AssetResourceApiResponse,  ResourceProps,  ResourceTableRow } from "@/types";
import { ColumnDef } from "@tanstack/react-table";
import { Container } from "@/components/Container";
import { DataTable } from "./data-table";

import { getResources } from "@/hooks/GetResources"; 



export default function ResourceOverview({asset}: ResourceProps) {
  console.log("Here")

  const {
    data,
    error,
    isLoading,
    refetch,
  } = getResources(asset);
  

  if (isLoading) {
    return <p>Loading...</p>
  }
  if (error) {
    return <p>Error: {error.message}</p>
  }


  console.log(data)
  const columns = makeColumns(data!);
  const tableData = makeTableRows(data!);

  return (
    <Container maxWidth="full" padding="sm" className="bg-white border border-gray-300 shadow rounded-lg">
      <DataTable asset={asset} columns={columns} data={tableData} onUpdate={refetch}/>
    </Container>
  );
}

function makeTableRows(apiData: AssetResourceApiResponse): ResourceTableRow[] {
  return apiData.data.map((entry, index) => {
    const periodObj = apiData.metadata.periods.find((p) => p.id === entry.periodId)
    const periodId = periodObj ? periodObj.id : entry.periodId
    const periodLabel = periodObj ? periodObj.label : entry.periodId

    return {
      id: `row-${index}`,
      periodId: periodId,
      periodLabel: periodLabel,
      ...entry.values,
    }
  })
}

function makeColumns(apiData: AssetResourceApiResponse): ColumnDef<ResourceTableRow>[] {
  console.log("Debug")
  const periodColumn: ColumnDef<ResourceTableRow> = {
    accessorKey: "periodLabel",
    header: "Period"
  }

  const resourceColumn: ColumnDef<ResourceTableRow>[] = apiData.metadata.resources.map((r) => ({
    accessorKey: r.id,
    header: r.label
  }))

  return [periodColumn, ...resourceColumn]
}
