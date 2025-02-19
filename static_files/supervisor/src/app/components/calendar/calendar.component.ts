import { Component, Input } from '@angular/core';
import { CalendarSectionComponent } from '../calendar-section/calendar-section.component';
import { CommonModule } from '@angular/common';
import { CalendarInstance, CalendarDataCollection } from '@app-types/types';

@Component({
  selector: 'app-calendar',
  standalone: true,
  imports: [CalendarSectionComponent, CommonModule],
  templateUrl: './calendar.component.html',
})
export class CalendarComponent {
  @Input() calendarData: CalendarDataCollection[] = [];
}
