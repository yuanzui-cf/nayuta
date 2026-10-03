import type { MessageKey } from '../../i18n';
export interface PostField {
  name: string;
  label: MessageKey;
  kind: 'text' | 'boolean' | 'tags' | 'license';
  required?: boolean;
}
export const postFields: readonly PostField[] = [
  { name: 'title', label: 'title', kind: 'text', required: true },
  { name: 'publishDate', label: 'publishDate', kind: 'text', required: true },
  { name: 'description', label: 'description', kind: 'text' },
  { name: 'cover', label: 'cover', kind: 'text' },
  { name: 'tags', label: 'tags', kind: 'tags' },
  { name: 'views', label: 'views', kind: 'text' },
  { name: 'draft', label: 'draft', kind: 'boolean' },
  { name: 'exclude_in_search', label: 'excludeInSearch', kind: 'boolean' },
  { name: 'withLeftSidebar', label: 'withLeftSidebar', kind: 'boolean' },
  { name: 'withProfileCard', label: 'withProfileCard', kind: 'boolean' },
  { name: 'withRightSidebar', label: 'withRightSidebar', kind: 'boolean' },
  { name: 'copyright.enabled', label: 'copyrightEnabled', kind: 'boolean' },
  { name: 'copyright.license', label: 'copyrightLicense', kind: 'license' },
];
