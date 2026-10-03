import { isMap, isScalar, Scalar, parseDocument, type Document } from 'yaml';
import { NayutaError } from '../errors';
import { validate } from '../blog/schema';
import { postSchema, type PostData } from './schema';
import { postFields } from './fields';

export interface FrontmatterSnapshot {
  path: string;
  source: string;
  body: string;
  bom: string;
  newline: string;
  closingNewline: string;
  document: Document;
  values: PostData;
}
export function parseFrontmatter(
  source: string,
  path: string,
): FrontmatterSnapshot {
  const match =
    /^(\uFEFF?)---[ \t]*(\r?\n)([\s\S]*?)\r?\n---[ \t]*(\r?\n|$)/.exec(source);
  if (!match)
    throw new NayutaError('errorFrontmatter', {
      path,
      detail: 'missingDelimiters',
    });
  const document = parseDocument(match[3]!, {
    version: '1.1',
    keepSourceTokens: true,
    uniqueKeys: true,
  });
  if (document.errors.length || !isMap(document.contents))
    throw new NayutaError('errorFrontmatter', {
      path,
      detail: document.errors[0]?.message ?? 'yamlMapping',
    });
  let data: unknown;
  try {
    data = document.toJS({ maxAliasCount: 100 });
  } catch (error) {
    throw new NayutaError('errorFrontmatter', { path, detail: String(error) });
  }
  const values = validate(postSchema, data);
  return {
    path,
    source,
    body: source.slice(match[0].length),
    bom: match[1]!,
    newline: match[2]!,
    closingNewline: match[4]!,
    document,
    values,
  };
}
export function prepareFrontmatter(
  snapshot: FrontmatterSnapshot,
  patch: Record<string, unknown>,
): string {
  const document = snapshot.document.clone();
  for (const [name, value] of Object.entries(patch)) {
    if (!postFields.some((field) => field.name === name))
      throw new NayutaError('errorValidation', {
        field: name,
        detail: 'unknownField',
      });
    const parts = name.split('.');
    if (value !== undefined && parts.length > 1 && !document.has(parts[0]!)) {
      document.set(parts[0]!, document.createNode({}));
    }
    if (value === undefined) document.deleteIn(parts);
    else
      document.setIn(
        parts,
        name === 'tags' && Array.isArray(value)
          ? [
              ...new Set(
                value.map((tag) =>
                  typeof tag === 'string' ? tag.trim() : tag,
                ),
              ),
            ]
          : value,
      );
  }
  const copyright = document.get('copyright', true);
  for (const name of Object.keys(patch)) {
    const node = document.getIn(name.split('.'), true);
    if (isScalar(node) && typeof node.value === 'string')
      node.type = Scalar.QUOTE_DOUBLE;
  }
  if (isMap(copyright) && copyright.items.length === 0)
    document.delete('copyright');
  validate(postSchema, document.toJS({ maxAliasCount: 100 }));
  const yaml = document
    .toString({ lineWidth: 0 })
    .replaceAll('\n', snapshot.newline);
  return `${snapshot.bom}---${snapshot.newline}${yaml}---${snapshot.closingNewline}${snapshot.body}`;
}
