import { Component, Input } from '@angular/core';
import { HlmButtonDirective } from '@spartan-ng/ui-button-helm';

@Component({
  selector: 'app-button-secondary',
  standalone: true,
  imports: [HlmButtonDirective],
  templateUrl: './button-secondary.component.html',
})
export class ButtonSecondary {
  @Input() text: string = '';
}
