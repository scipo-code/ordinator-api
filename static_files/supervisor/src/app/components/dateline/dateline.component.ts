import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DateService } from '../../services/date-service';

@Component({
  selector: 'app-dateline',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './dateline.component.html',
})
export class DatelineComponent implements OnInit {
  dates: Date[] = [];
  selectedDate: Date | null = null;
  private currentDate: Date = new Date(); // Store the current date

  constructor(private dateService: DateService) {
    this.generateDates();
    // Subscribe to the selectedDate$ observable to update hours when the date changes

    this.dateService.selectedDate$.subscribe((date) => {
      this.selectedDate = date;
    });
  }

  ngOnInit() {
    this.dateService.setSelectedDate(this.currentDate);
  }

  generateDates() {
    // Generate the next 12 days starting from the current date (and 07:00 in the morning)
    this.dates = Array.from({ length: 7 }, (_, i) => {
      const date = new Date(this.currentDate);
      date.setHours(7, 0, 0, 0);
      date.setDate(this.currentDate.getDate() + i); // Add i days to the current date
      return date;
    });
  }

  onDateClick(date: Date) {
    this.selectedDate = date;
    this.dateService.setSelectedDate(date); // Notify the service of the selected date
    date.setHours(7, 0, 0, 0);
  }

  changeWeek(weeks: number) {
    // Update the current date by adding weeks (7 days)
    this.currentDate.setDate(this.currentDate.getDate() + weeks * 7);
    this.generateDates(); // Regenerate dates based on the updated current date
    // Run onDateClick for the first day of the new generated week
    if (this.dates && this.dates.length > 0) {
      this.onDateClick(this.dates[0]); // Select the first day of the new week
    }

    // run the OnDateClick, for the date of the first day in the new generated week:
  }
}
