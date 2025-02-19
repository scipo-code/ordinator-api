import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { CalendarSectionComponent } from './components/calendar-section/calendar-section.component';
import { CalendarItemComponent } from './components/calendar-item/calendar-item.component';
import { TimelineComponent } from './components/timeline/timeline.component';
import { CalendarBannerComponent } from './components/calendar-banner/calendar-banner.component';
import { CalendarComponent } from './components/calendar/calendar.component';
import { CalendarViewSelectionComponent } from './components/calendar-view-selection/calendar-view-selection.component';
import { SidebarComponent } from './components/sidebar/sidebar.component';
import { ButtonPrimary } from './components/button-primary/button-primary.component';
import { ButtonSecondary } from './components/button-secondary/button-secondary.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    RouterOutlet,
    CalendarSectionComponent,
    CalendarItemComponent,
    TimelineComponent,
    CalendarBannerComponent,
    CalendarComponent,
    CalendarViewSelectionComponent,
    SidebarComponent,
    ButtonPrimary,
    ButtonSecondary,
  ],
  templateUrl: './app.component.html',
})
export class AppComponent {
  title = 'supervisor-calendar';
}
