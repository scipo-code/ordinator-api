import { Injectable } from '@angular/core';
import { BehaviorSubject } from 'rxjs';
import {
  CalendarDataCollection,
  CalendarInstance,
  regularHours,
  overTimeHours,
} from '@app-types/types';
@Injectable({
  providedIn: 'root',
})
export class CalendarDataService {
  //constructor() {
  //  this.continuesUpdateCalendarDataCollection();
  //}

  constructor() {
    this.generateMockData();
  }

  calendarDataCollection: CalendarDataCollection[] = [];

  private filteredCalendarData = new BehaviorSubject<
    CalendarDataCollection[] | null
  >(null);
  filteredCalendarData$ = this.filteredCalendarData.asObservable();

  setFilteredCalendarData(calendarData: CalendarDataCollection[]): void {
    this.calendarDataCollection = calendarData;
    this.filteredCalendarData.next(calendarData.length ? calendarData : null);
  }

  generateMockData() {
    this.calendarDataCollection = [
      {
        calendarData: [
          {
            banner: 'Johan Sne',
            items: [
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 7, 15)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 8, 45)).toISOString(),
                color: 'bg-blue-400',
              },
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 9, 23)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 10, 34)).toISOString(),
                color: 'bg-red-400',
              },
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 21, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 23, 0)).toISOString(),
                color: 'bg-blue-400',
              },
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 23, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 5, 2, 40)).toISOString(),
                color: 'bg-blue-600',
              },
            ],
          },
        ],
      },
      {
        calendarData: [
          {
            banner: 'Ole Hansen',
            items: [
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 7, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 8, 0)).toISOString(),
                color: 'bg-green-400',
              },
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 9, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 10, 0)).toISOString(),
                color: 'bg-green-400',
              },
            ],
          },
        ],
      },
      {
        calendarData: [
          {
            banner: 'Ole Mortensen',
            items: [
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 7, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 8, 0)).toISOString(),
                color: 'bg-green-400',
              },
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 9, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 10, 0)).toISOString(),
                color: 'bg-green-400',
              },
            ],
          },
        ],
      },
      {
        calendarData: [
          {
            banner: 'Mortensen Emil',
            items: [
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 7, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 8, 0)).toISOString(),
                color: 'bg-green-400',
              },
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 9, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 10, 0)).toISOString(),
                color: 'bg-green-400',
              },
            ],
          },
        ],
      },
      {
        calendarData: [
          {
            banner: 'Mads Jensen',
            items: [
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 10, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 11, 0)).toISOString(),
                color: 'bg-yellow-400',
              },
              {
                startDate: new Date(Date.UTC(2024, 10, 4, 12, 0)).toISOString(),
                endDate: new Date(Date.UTC(2024, 10, 4, 13, 0)).toISOString(),
                color: 'bg-yellow-400',
              },
            ],
          },
        ],
      },
    ];
  }

  continuesUpdateCalendarDataCollection() {
    setInterval(() => {
      this.generateMockData(); // Replace with actual API call later
    }, 10000); // 10 seconds
  }

  updateCalendarData(selectedDate: Date | null = null) {
    this.filterByToDay(selectedDate);
  }

  filterByToDay(selectedDate: Date | null = null) {
    if (selectedDate == null) {
      console.error('selectedDate was null');
      return;
    }

    if (regularHours.includes(selectedDate.getHours())) {
      const CropFrom = new Date(selectedDate);
      CropFrom.setHours(7, 0, 0, 0);
      console.log('CropFrom (Regular Hours):', new Date(CropFrom));
      const CropTo = new Date(selectedDate);
      CropTo.setHours(19, 0, 0, 0);
      console.log('CropTo (Regular Hours):', new Date(CropTo));
      this.filterCalendarCollection(CropFrom, CropTo);
    }

    if (overTimeHours.includes(selectedDate.getHours())) {
      const CropFrom = new Date(selectedDate);
      CropFrom.setHours(19, 0, 0, 0);
      console.log('CropFrom (Overtime Hours):', new Date(CropFrom));
      const CropTo = new Date(selectedDate);
      CropTo.setDate(selectedDate.getDate() + 1);
      CropTo.setHours(7, 0, 0, 0);
      console.log('CropTo (Overtime Hours):', new Date(CropTo));
      this.filterCalendarCollection(CropFrom, CropTo);
    }
  }

  filterCalendarCollection(CropFrom: Date, CropTo: Date) {
    const filteredData = this.calendarDataCollection
      .map((entry) => {
        // Filter items within each CalendarInstance
        const filteredCalendarData = entry.calendarData
          .map((instance) => {
            const filteredItems = instance.items.filter((item) => {
              // Convert item startDate and endDate to Date objects
              const itemStartDate = new Date(item.startDate);
              const itemEndDate = new Date(item.endDate);

              console.log('CropFrom:', CropFrom);
              console.log('CropTo:', CropTo);
              console.log('Item Start:', itemStartDate);
              console.log('Item End:', itemEndDate);

              // Check for any overlap with the CropFrom and CropTo range
              return itemEndDate > CropFrom && itemStartDate < CropTo;
            });

            // Return a new CalendarInstance with filtered items
            return { ...instance, items: filteredItems };
          })
          .filter((instance) => instance.items.length > 0); // Keep instances with at least one item
        return { ...entry, calendarData: filteredCalendarData };
      })
      .filter((entry) => entry.calendarData.length > 0); // Keep entries with at least one instance

    // Assuming `filteredCalendarData` is a BehaviorSubject
    this.filteredCalendarData.next(filteredData.length ? filteredData : null);
  }

  formatTime(date: Date, time: string) {
    // Create a new Date instance to avoid modifying the original date
    const newDate = new Date(date);
    const [hours, minutes] = time.split(':').map(Number);
    newDate.setHours(hours, minutes, 0, 0);
    return newDate;
  }
}
