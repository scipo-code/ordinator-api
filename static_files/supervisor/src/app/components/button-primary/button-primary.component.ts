import { Component, Input } from '@angular/core';
import { HlmButtonDirective } from '@spartan-ng/ui-button-helm';

@Component({
  selector: 'app-button-primary',
  standalone: true,
  imports: [HlmButtonDirective],
  templateUrl: './button-primary.component.html',
})
export class ButtonPrimary {
  @Input() text: string = '';
}
