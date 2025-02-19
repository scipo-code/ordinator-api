import { Injectable } from '@angular/core';
import { DateService } from './date-service';
@Injectable({
  providedIn: 'root',
})
export class TimeGridService {
  constructor(private dateService: DateService) {}

  getGridPosition(startDate: string, endDate: string) {
    // Use getHour and getMinute to extract hour and minute values from the dates
    const startHour = this.getHour(startDate);
    const startMinute = this.getMinute(startDate);

    const isMorningRange = startHour >= 7 && startHour < 19;
    const baseHour = isMorningRange ? 7 : 19; // Use 6 AM or 6 PM as the base hour

    const start = new Date(startDate);
    const end = new Date(endDate);
    const durationMinutes = this.calculateDurationInMinutes(start, end);

    console.log('dur', durationMinutes);

    const left =
      ((startHour - baseHour) * 100) / 12 + (startMinute / 60) * (100 / 12); // relative to a 12-hour span, res in % of much left to go.

    // Calculate width based on duration
    const width = (durationMinutes / 60) * (100 / 12); // how wide (long), should the item be?

    return {
      left: `${left}%`,
      width: `${width}%`,
    };
  }

  // Extract hour from time in ISO format
  private getHour(time: string): number {
    const date = new Date(time);
    return date.getUTCHours();
  }

  // Extract minute from time in ISO format
  private getMinute(time: string): number {
    const date = new Date(time);
    return date.getUTCMinutes();
  }

  private calculateDurationInMinutes(startDate: Date, endDate: Date): number {
    const startHour = startDate.getUTCHours();
    const startMinute = startDate.getUTCMinutes();

    const endHour = endDate.getUTCHours();
    const endMinute = endDate.getUTCMinutes();

    // Check if the event spans across two days
    if (endDate.getUTCDate() !== startDate.getUTCDate()) {
      // Calculate duration spanning midnight
      const minutesUntilMidnight = (24 - startHour) * 60 - startMinute;
      const minutesAfterMidnight = endHour * 60 + endMinute;
      return minutesUntilMidnight + minutesAfterMidnight;
    } else {
      // Calculate regular duration within the same day
      return endHour * 60 + endMinute - (startHour * 60 + startMinute);
    }
  }
}
