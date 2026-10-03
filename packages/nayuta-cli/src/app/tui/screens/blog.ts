import { resolve } from 'node:path';
import type { AppContext } from '../../context';
import { createBlog } from '../../../core/blog/service';
import {
  themes,
  licenses,
  today,
  type SiteSettings,
} from '../../../core/blog/schema';
import { discoverProject } from '../../../core/project/discover';
import {
  prepareSiteConfig,
  readSiteConfig,
  writeSiteConfig,
} from '../../../infrastructure/site-config';
import {
  defaultTemplate,
  defaultTemplateRef,
} from '../../../infrastructure/template';
import { gitIdentity } from '../../../infrastructure/git';
import type { MessageKey } from '../../../i18n';
import { textDiff } from '../components/diff';
import { changedFields, type FormField, type FormModel } from '../state';

function siteFields(
  context: AppContext,
  settings: SiteSettings,
  dynamic: string[] = [],
): FormField[] {
  const labels: Record<string, MessageKey> = {
    title: 'title',
    author: 'author',
    avatar: 'avatar',
    description: 'description',
    site_url: 'siteUrl',
    theme: 'theme',
    since: 'since',
    postsPerPage: 'postsPerPage',
    links: 'links',
  };
  const fields: FormField[] = Object.entries(labels)
    .filter(([name]) => !dynamic.includes(name))
    .map(([name, label]) => {
      const value = settings[name as keyof SiteSettings];
      return {
        name,
        label,
        value:
          name === 'links' ? JSON.stringify(value ?? []) : String(value ?? ''),
        choices:
          name === 'theme'
            ? themes.map((theme) => ({ value: theme, label: theme }))
            : undefined,
      };
    });
  if (!dynamic.includes('copyright')) {
    fields.push({
      name: 'copyright.enabled',
      label: 'copyrightEnabled',
      value: String(settings.copyright?.enabled ?? false),
      choices: [
        { value: 'true', label: context.i18n.t('enabled') },
        { value: 'false', label: context.i18n.t('disabled') },
      ],
    });
    fields.push({
      name: 'copyright.license',
      label: 'copyrightLicense',
      value: settings.copyright?.license ?? '',
      choices: [
        ...(settings.copyright?.license
          ? []
          : [{ value: '', label: context.i18n.t('disabled') }]),
        ...licenses.map((license) => ({ value: license, label: license })),
      ],
    });
  }
  return fields;
}
function settingsFromValues(values: Record<string, string>): SiteSettings {
  const settings: Record<string, unknown> = {};
  for (const [name, value] of Object.entries(values)) {
    if (name.startsWith('copyright.')) {
      const copyright = (settings.copyright ??= {}) as Record<string, unknown>;
      if (name === 'copyright.enabled') copyright.enabled = value === 'true';
      else if (value) copyright.license = value;
    } else if (name === 'links') settings.links = JSON.parse(value || '[]');
    else if (name === 'postsPerPage') {
      if (value) settings[name] = Number(value);
    } else if (value || name === 'description') settings[name] = value;
  }
  return settings as SiteSettings;
}
export async function blogSetupForm(context: AppContext): Promise<FormModel> {
  const root = await discoverProject(context.cwd);
  const snapshot = await readSiteConfig(root);
  const fields = siteFields(context, snapshot.values, snapshot.dynamic);
  const initial = Object.fromEntries(
    fields.map((field) => [field.name, field.value]),
  );
  return {
    title: 'setupBlog',
    fields,
    prepare: async (values) => {
      const updates = Object.fromEntries(
        changedFields(initial, values).map((name) => [name, values[name]!]),
      );
      const content = prepareSiteConfig(snapshot, settingsFromValues(updates));
      return {
        preview: textDiff(snapshot.path, snapshot.source, content),
        save: async () => {
          await writeSiteConfig(snapshot, content);
          return { path: snapshot.path };
        },
      };
    },
  };
}
export async function blogCreateForm(context: AppContext): Promise<FormModel> {
  const t = context.i18n.t.bind(context.i18n);
  let author = '';
  try {
    author = (await gitIdentity(context.cwd)).name;
  } catch {
    /* The user can enter a profile name. */
  }
  const yesNo = [
    { value: 'true', label: t('yes') },
    { value: 'false', label: t('no') },
  ];
  const fields: FormField[] = [
    { name: 'directory', label: 'directory', value: './my-blog' },
    ...siteFields(context, {
      title: 'My Blog',
      author,
      site_url: 'https://example.com',
      avatar: '/assets/images/avatar.png',
      theme: 'nayuta',
      since: today(),
      postsPerPage: 10,
      description: '',
      links: [{ text: t('postsLink'), url: '/posts' }],
      copyright: { enabled: false },
    }),
    { name: 'template', label: 'template', value: defaultTemplate },
    { name: 'templateRef', label: 'templateRef', value: defaultTemplateRef },
    { name: 'install', label: 'install', value: 'true', choices: yesNo },
    { name: 'commit', label: 'commit', value: 'true', choices: yesNo },
    { name: 'githubRepo', label: 'githubRepo', value: '' },
    {
      name: 'visibility',
      label: 'visibility',
      value: 'private',
      choices: [
        { value: 'private', label: t('private') },
        { value: 'public', label: t('public') },
      ],
    },
    { name: 'push', label: 'push', value: 'false', choices: yesNo },
  ];
  return {
    title: 'createBlog',
    fields,
    prepare: async (values) => {
      const siteNames = siteFields(context, {}).map((field) => field.name);
      const settings = settingsFromValues(
        Object.fromEntries(siteNames.map((name) => [name, values[name]!])),
      );
      const preview = fields
        .map(
          (field) =>
            `${t(field.label)}: ${field.choices?.find((choice) => choice.value === values[field.name])?.label ?? values[field.name]}`,
        )
        .join('\n');
      return {
        preview,
        save: async () => {
          const result = await createBlog({
            directory: resolve(context.cwd, values.directory!),
            settings,
            i18n: context.i18n,
            template: values.template,
            templateRef: values.templateRef,
            install: values.install === 'true',
            commit: values.commit === 'true',
            githubRepo: values.githubRepo || undefined,
            visibility: values.visibility as 'private' | 'public',
            push: values.push === 'true',
          });
          context.cwd = result.path;
          return result;
        },
      };
    },
  };
}
