import { ResourcePeriod, ResourceTableRow } from "@/types";

export function transformResourcePeriodToResourceRow(data: ResourcePeriod[]): ResourceTableRow[] {
  const allWorkCenters = Array.from(
    new Set(data.flatMap((entry) => entry.workCenters.map((wc) => wc.workCenter)))
  );

  return data.map((entry, index) => {
    const row: ResourceTableRow = { id: `row-${index}`, period: entry.period };

    allWorkCenters.forEach((workCenter) => {
      const found = entry.workCenters.find((wc) => wc.workCenter === workCenter);
      row[workCenter] = found ? found.noOfTech : 0;
    });

    return row
  })
}
