import type { MessageKey } from '../../i18n';
export interface Choice {
  value: string;
  label: string;
}
export interface FormField {
  name: string;
  label: MessageKey;
  value: string;
  choices?: Choice[];
}
export interface PreparedAction {
  preview: string;
  save: () => Promise<{ path: string }>;
}
export interface FormModel {
  title: MessageKey;
  fields: FormField[];
  prepare: (values: Record<string, string>) => Promise<PreparedAction>;
}
export type Screen =
  'home' | 'form' | 'review' | 'posts' | 'settings' | 'confirm';
export function changedFields(
  initial: Record<string, string>,
  current: Record<string, string>,
): string[] {
  return Object.keys(current).filter((name) => current[name] !== initial[name]);
}
