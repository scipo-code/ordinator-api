import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { CalendarBannerComponent } from '../calendar-banner/calendar-banner.component';
import { CalendarItemComponent } from '../calendar-item/calendar-item.component';
import { CalendarInstance, CalendarDataCollection } from '@app-types/types';
@Component({
  selector: 'app-calendar-section',
  standalone: true,
  imports: [CommonModule, CalendarBannerComponent, CalendarItemComponent],
  templateUrl: './calendar-section.component.html',
})
export class CalendarSectionComponent {
  @Input() calendarData!: CalendarInstance;
}
