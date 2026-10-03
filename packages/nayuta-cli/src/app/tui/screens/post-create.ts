import type { AppContext } from '../../context';
import { createPost } from '../../../core/post/service';
import { today } from '../../../core/blog/schema';
import type { FormField, FormModel } from '../state';
export function postCreateForm(context: AppContext): FormModel {
  const t = context.i18n.t.bind(context.i18n);
  const yesNo = [
    { value: 'true', label: t('yes') },
    { value: 'false', label: t('no') },
  ];
  const fields: FormField[] = [
    { name: 'title', label: 'title', value: '' },
    { name: 'slug', label: 'slug', value: '' },
    { name: 'publishDate', label: 'publishDate', value: today() },
    {
      name: 'format',
      label: 'format',
      value: 'md',
      choices: [
        { value: 'md', label: 'Markdown' },
        { value: 'mdx', label: 'MDX' },
      ],
    },
    { name: 'description', label: 'description', value: '' },
    { name: 'tags', label: 'tags', value: '' },
    { name: 'cover', label: 'cover', value: '' },
    { name: 'draft', label: 'draft', value: 'true', choices: yesNo },
    { name: 'datePrefix', label: 'datePrefix', value: 'true', choices: yesNo },
  ];
  return {
    title: 'createPost',
    fields,
    prepare: async (values) => ({
      preview: fields
        .map(
          (field) =>
            `${t(field.label)}: ${field.choices?.find((choice) => choice.value === values[field.name])?.label ?? values[field.name]}`,
        )
        .join('\n'),
      save: () =>
        createPost({
          directory: context.cwd,
          i18n: context.i18n,
          title: values.title!,
          slug: values.slug || undefined,
          format: values.format as 'md' | 'mdx',
          publishDate: values.publishDate,
          description: values.description || undefined,
          tags: values.tags ? values.tags.split(',') : [],
          cover: values.cover || undefined,
          published: values.draft === 'false',
          datePrefix: values.datePrefix !== 'false',
          assetBase: context.cwd,
        }),
    }),
  };
}
