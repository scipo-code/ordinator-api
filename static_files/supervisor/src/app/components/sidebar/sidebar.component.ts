import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ButtonPrimary } from '../button-primary/button-primary.component';
import { ButtonSecondary } from '../button-secondary/button-secondary.component';

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [CommonModule, ButtonPrimary, ButtonSecondary], // Add CommonModule here
  templateUrl: './sidebar.component.html',
})
export class SidebarComponent {
  dashboardOpen = true;
  eCommerceOpen = false;

  toggleDashboard() {
    this.dashboardOpen = !this.dashboardOpen;
  }

  toggleECommerce() {
    this.eCommerceOpen = !this.eCommerceOpen;
  }
}
