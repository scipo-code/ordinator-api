import axios from "axios";
import { useState, useEffect } from "react";
import { Container } from "@/components/Container";
import { useParams, Navigate } from "react-router-dom";
import { AssetResourceApiResponse,  ResourceTableRow } from "@/types";
import { DataTable } from "./data-table";
import { ColumnDef } from "@tanstack/react-table";


export default function ResourceOverview() {
  const { asset } = useParams<{ asset: string}>();

  if (!asset) {
    return <Navigate to="/404" replace />
  }

  const [ tableData, setTableData ] = useState<ResourceTableRow[]>([]);
  const [ columns, setColumns ] = useState<ColumnDef<ResourceTableRow>[]>([]);

  const fetchData = async () => {
    try {
      const { data } = await axios.get<AssetResourceApiResponse>(`/api/scheduler/${asset}/resources`);
      const rows = makeTableRows(data);

      const cols = makeColumns(data);

      setTableData(rows);
      setColumns(cols);
    
    } catch (err) {
      console.error("Error fetching resources: ", err);
    }
  };

  useEffect(() => {
    fetchData()
  }, [asset])

  // TODO: Better error handling for when data is not available.
  return (
    <Container maxWidth="full" padding="sm" className="bg-white border border-gray-300 shadow rounded-lg">
      <DataTable asset={asset} columns={columns} data={tableData} onUpdate={fetchData}/>
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
