import { stringify } from 'yaml';
import type { I18n } from '../i18n';
import type { PostData } from '../core/post/schema';
export function renderPost(data: PostData, i18n: I18n): string {
  return `---\n${stringify(data, { version: '1.1' })}---\n\n## ${i18n.t('title')}\n\n${i18n.t('postBody')}\n`;
}
