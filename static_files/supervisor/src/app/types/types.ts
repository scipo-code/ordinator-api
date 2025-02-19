export type Hours = number[];

export const regularHours = [7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
export const overTimeHours = [19, 20, 21, 22, 23, 0, 1, 2, 3, 4, 5, 6];
export type Regular = typeof regularHours;
export type OverTime = typeof overTimeHours;

export type WorkHoursCategory = {
  regular: Regular;
  overTime: OverTime;
};

export type CalendarInstance = {
  banner: string;
  items: { startDate: string; endDate: string; color: string }[];
};

export type CalendarDataCollection = {
  calendarData: CalendarInstance[];
};
