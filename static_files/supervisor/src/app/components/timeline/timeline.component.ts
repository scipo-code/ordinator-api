import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DateService } from '../../services/date-service';
import { Hours } from '@app-types/types';

@Component({
  selector: 'app-timeline',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './timeline.component.html',
})
export class TimelineComponent {
  hours: Hours = []; // Initialize as an empty array

  constructor(private dateService: DateService) {
    // Subscribe to the selectedDate$ observable to update hours when the date changes
    this.dateService.selectedDate$.subscribe(() => {
      this.updateHours(); // Update hours when the date changes
    });
  }

  // Method to update the hours based on the selected date
  private updateHours() {
    this.hours = this.dateService.getHours(); // Update hours from the service
  }

  //private updateWorkTimeCategory() {
  //  this.hours = this.dateService.updateWorkTimeCategory;
  //}

  addHours(hours: number) {
    this.dateService.addHours(hours); // Add hours using the DateService
  }

  ngOnInit() {
    // Reset to morning hours when the component initializes
    this.updateHours(); // Set initial hours based on the reset date
  }
}
