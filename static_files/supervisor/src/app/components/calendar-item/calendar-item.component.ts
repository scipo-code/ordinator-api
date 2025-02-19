import { Component, Input, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TimeGridService } from '../../services/time-grid.service';

@Component({
  selector: 'app-calendar-item',
  standalone: true,
  templateUrl: './calendar-item.component.html',
  imports: [CommonModule], // Import CommonModule for ngStyle
})
export class CalendarItemComponent implements OnInit {
  @Input() startDate!: string;
  @Input() endDate!: string;
  @Input() color!: string;
  left!: string;
  width!: string;

  constructor(private timeGridService: TimeGridService) {}

  ngOnInit() {
    console.log('Start Date:', this.startDate);
    console.log('End Date:', this.endDate);
    console.log('Color:', this.color);
    const gridPosition = this.timeGridService.getGridPosition(
      this.startDate,
      this.endDate
    );
    this.left = gridPosition.left;
    this.width = gridPosition.width;
    console.log('Grid Position:', gridPosition);
  }
}
