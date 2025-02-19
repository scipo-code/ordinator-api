import { Injectable } from '@angular/core';
import { BehaviorSubject } from 'rxjs';
import {
  WorkHoursCategory,
  regularHours,
  overTimeHours,
  Hours,
} from '@app-types/types';

@Injectable({
  providedIn: 'root',
})
export class DateService {
  // Observable variable holding data to be shown in view (the selected calendars)
  private selectedDateSource = new BehaviorSubject<Date | null>(null);
  selectedDate$ = this.selectedDateSource.asObservable();

  setSelectedDate(date: Date) {
    this.selectedDateSource.next(date);
  }

  // Observable variable holding data about dateline shown
  private workHoursCategorySource =
    new BehaviorSubject<WorkHoursCategory | null>(null);
  workHoursCategory$ = this.workHoursCategorySource.asObservable();

  setWorkHoursCategory(hoursInView: WorkHoursCategory) {
    this.workHoursCategorySource.next(hoursInView);
  }

  // Methods for plumbing
  addHours(hours: number) {
    const currentDate = this.selectedDateSource.getValue();
    if (currentDate) {
      const newDate = new Date(currentDate);
      newDate.setHours(currentDate.getHours() + hours);
      this.setSelectedDate(newDate);
    }
  }

  getHours(): Hours {
    const selectedDate = this.selectedDateSource.getValue();
    if (selectedDate) {
      const hours = selectedDate.getHours();

      if (regularHours.includes(hours)) {
        return regularHours;
      } else if (overTimeHours.includes(hours)) {
        return overTimeHours;
      }
    }
    return [];
  }

  UtcToYMD(utc: string): Date {
    const date = new Date(utc);
    date.setUTCHours(0, 0, 0, 0);
    return date;
  }

  UtcToYMDH(utc: string): Date {
    const date = new Date(utc);
    date.setUTCMinutes(0, 0, 0);
    return date;
  }

  UtcToYMDHM(utc: string): Date {
    const date = new Date(utc);
    date.setUTCSeconds(0, 0);
    return date;
  }
}
