import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { CalendarComponent } from '../calendar/calendar.component';
import { DateService } from '../../services/date-service';
import { DatelineComponent } from '../dateline/dateline.component';
import { TimelineComponent } from '../timeline/timeline.component';
import { CalendarDataService } from '../../services/calendar-data.service';
import { CalendarInstance, CalendarDataCollection } from '@app-types/types';

@Component({
  selector: 'app-calendar-view-selection',
  standalone: true,
  imports: [
    CommonModule,
    CalendarComponent,
    DatelineComponent,
    TimelineComponent,
  ],
  templateUrl: './calendar-view-selection.component.html',
})
export class CalendarViewSelectionComponent implements OnInit {
  constructor(
    private dateService: DateService,
    private calendarDataService: CalendarDataService
  ) {}
  selectedDate: Date | null = null;
  dateOptions: { date: Date; label: string }[] = [];
  selectedCalendarData: CalendarDataCollection[] | null = null;

  ngOnInit() {
    // Subscribe to the selected date observable
    this.dateService.selectedDate$.subscribe((date) => {
      this.selectedDate = date;
      if (this.selectedDate) {
        this.calendarDataService.updateCalendarData(this.selectedDate);
      }
    });

    // Subscribe to filtered calendar data from the CalendarDataService
    this.calendarDataService.filteredCalendarData$.subscribe((data) => {
      this.selectedCalendarData = data;
      console.log('thisdat', this.selectedCalendarData);
    });
  }
}
