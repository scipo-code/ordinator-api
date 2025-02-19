import { ResourceTableRow } from "@/types";
import { ColumnDef } from "@tanstack/react-table";


export function createColumnsFromResourceData(data: ResourceTableRow[]): ColumnDef<ResourceTableRow>[] {
  if (data.length === 0) {
    throw new Error("No data provided");
  }

  const firstRow = data[0];

  return Object.keys(firstRow).filter((key) => key !== "id").map((key) => {
    if (key === "period") {
      return {
        accessorKey: key,
        header: "Period",
      };
    }

    return {
      accessorKey: key,
      header: key,
    };
  });
  
}
