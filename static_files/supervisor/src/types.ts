export type Asset = {
  asset: string;
};

export type WorkCenter = { workCenter: string; noOfTech: number };
export type ResourcePeriod = { period: string; workCenters: WorkCenter[] };

export type ResourceTableRow = {
  id: string;
  periodId: string;
  periodLabel: string;
  [workCenter: string]: number | string;
};
