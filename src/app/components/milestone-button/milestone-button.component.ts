import { Component, output } from '@angular/core';

@Component({
  selector: 'app-milestone-button',
  standalone: true,
  templateUrl: './milestone-button.component.html',
  styleUrl: './milestone-button.component.scss',
})
export class MilestoneButtonComponent {
  milestone = output<void>();

  onClick() {
    this.milestone.emit();
  }
}
