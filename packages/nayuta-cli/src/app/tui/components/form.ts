import {
  BoxRenderable,
  InputRenderable,
  InputRenderableEvents,
  SelectRenderable,
  SelectRenderableEvents,
  ScrollBoxRenderable,
  type CliRenderer,
} from '@opentui/core';
import type { I18n } from '../../../i18n';
import type { FormField } from '../state';
import { theme } from '../theme';

export class FormView {
  readonly root: ScrollBoxRenderable;
  private widgets: (InputRenderable | SelectRenderable)[] = [];
  private rows: BoxRenderable[] = [];
  private index = 0;
  constructor(
    renderer: CliRenderer,
    fields: FormField[],
    i18n: I18n,
    values: Record<string, string>,
  ) {
    this.root = new ScrollBoxRenderable(renderer, {
      id: 'form-scroll',
      flexGrow: 1,
      scrollY: true,
      scrollX: false,
      contentOptions: { flexDirection: 'column', gap: 1 },
    });
    for (const field of fields) {
      const row = new BoxRenderable(renderer, {
        id: `row-${field.name}`,
        height: 3,
        flexShrink: 0,
        border: true,
        title: i18n.t(field.label),
        borderColor: theme.muted,
        focusedBorderColor: theme.accent,
      });
      const widget = field.choices
        ? new SelectRenderable(renderer, {
            id: `field-${field.name}`,
            height: 1,
            width: '100%',
            showDescription: false,
            backgroundColor: theme.background,
            textColor: theme.text,
            selectedBackgroundColor: theme.selected,
            options: field.choices.map((choice) => ({
              name: choice.label,
              value: choice.value,
              description: '',
            })),
            selectedIndex: Math.max(
              0,
              field.choices.findIndex(
                (choice) => choice.value === values[field.name],
              ),
            ),
          })
        : new InputRenderable(renderer, {
            id: `field-${field.name}`,
            width: '100%',
            value: values[field.name] ?? field.value,
            backgroundColor: theme.background,
            textColor: theme.text,
            focusedBackgroundColor: theme.selected,
          });
      widget.on(
        field.choices
          ? SelectRenderableEvents.ITEM_SELECTED
          : InputRenderableEvents.ENTER,
        () => this.move(1),
      );
      row.add(widget);
      this.root.add(row);
      this.rows.push(row);
      this.widgets.push(widget);
    }
  }
  focus(): void {
    this.widgets[this.index]?.focus();
    if (this.rows[this.index])
      this.root.scrollChildIntoView(this.rows[this.index]!.id);
  }
  move(offset: number): void {
    if (!this.widgets.length) return;
    const focused = this.widgets.findIndex((widget) => widget.focused);
    if (focused >= 0) this.index = focused;
    this.widgets[this.index]?.blur();
    this.index =
      (this.index + offset + this.widgets.length) % this.widgets.length;
    this.focus();
  }
  values(fields: FormField[]): Record<string, string> {
    return Object.fromEntries(
      fields.map((field, index) => {
        const widget = this.widgets[index]!;
        const value =
          widget instanceof InputRenderable
            ? widget.value
            : String(widget.getSelectedOption()?.value ?? '');
        return [field.name, value];
      }),
    );
  }
}
