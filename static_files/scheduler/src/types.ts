export type Asset = { value: string; label: string };


export type WorkCenter = { workCenter: string; noOfTech: number };
export type ResourcePeriod = { period: string; workCenters: WorkCenter[] }

export type ResourceTableRow = {
  id: string,
  period: string,
  [workCenter: string]: number | string
  
}
