import ts from 'typescript';
import { join } from 'node:path';
import { NayutaError } from '../core/errors';
import {
  siteSettingsSchema,
  validate,
  type SiteSettings,
} from '../core/blog/schema';
import { assertInside, atomicWrite, readText } from './filesystem';
function propertyName(node: ts.PropertyName): string | undefined {
  return ts.isIdentifier(node) || ts.isStringLiteral(node)
    ? node.text
    : undefined;
}
function configObject(source: ts.SourceFile): ts.ObjectLiteralExpression {
  let exported: ts.Expression | undefined;
  for (const statement of source.statements)
    if (ts.isExportAssignment(statement) && !statement.isExportEquals)
      exported = statement.expression;
  if (exported && ts.isIdentifier(exported)) {
    const name = exported.text;
    for (const statement of source.statements) {
      if (!ts.isVariableStatement(statement)) continue;
      for (const declaration of statement.declarationList.declarations) {
        if (ts.isIdentifier(declaration.name) && declaration.name.text === name)
          exported = declaration.initializer;
      }
    }
  }
  while (
    exported &&
    (ts.isSatisfiesExpression(exported) ||
      ts.isAsExpression(exported) ||
      ts.isParenthesizedExpression(exported))
  )
    exported = exported.expression;
  if (!exported || !ts.isObjectLiteralExpression(exported))
    throw new NayutaError('errorConfig', { field: 'default export' });
  return exported;
}
function literal(node: ts.Expression): unknown {
  if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node))
    return node.text;
  if (ts.isNumericLiteral(node)) return Number(node.text);
  if (node.kind === ts.SyntaxKind.TrueKeyword) return true;
  if (node.kind === ts.SyntaxKind.FalseKeyword) return false;
  if (node.kind === ts.SyntaxKind.NullKeyword) return null;
  if (ts.isArrayLiteralExpression(node))
    return node.elements.map((element) => literal(element));
  if (ts.isObjectLiteralExpression(node))
    return Object.fromEntries(
      node.properties.map((property) => {
        if (!ts.isPropertyAssignment(property) || !propertyName(property.name))
          throw new Error('dynamic');
        return [propertyName(property.name)!, literal(property.initializer)];
      }),
    );
  throw new Error('dynamic');
}
export interface SiteConfigSnapshot {
  path: string;
  source: string;
  values: SiteSettings;
  dynamic: string[];
}
export async function readSiteConfig(
  root: string,
): Promise<SiteConfigSnapshot> {
  const path = join(root, 'src/config.ts');
  await assertInside(root, path);
  const source = await readText(path);
  const parsed = ts.createSourceFile(
    path,
    source,
    ts.ScriptTarget.Latest,
    true,
  );
  const object = configObject(parsed);
  const values: Record<string, unknown> = {};
  const dynamic: string[] = [];
  for (const property of object.properties) {
    const name = property.name && propertyName(property.name);
    if (!name || !(name in siteSettingsSchema.shape)) continue;
    try {
      if (!ts.isPropertyAssignment(property)) throw new Error('dynamic');
      values[name] = literal(property.initializer);
    } catch {
      dynamic.push(name);
    }
  }
  return { path, source, values: values as SiteSettings, dynamic };
}
export function prepareSiteConfig(
  snapshot: SiteConfigSnapshot,
  input: SiteSettings,
): string {
  const updates = validate(siteSettingsSchema, input);
  const source = ts.createSourceFile(
    snapshot.path,
    snapshot.source,
    ts.ScriptTarget.Latest,
    true,
  );
  const object = configObject(source);
  if (object.properties.some(ts.isSpreadAssignment))
    throw new NayutaError('errorConfig', { field: 'spread' });
  const edits: { start: number; end: number; text: string }[] = [];
  const additions: string[] = [];
  for (const [name, value] of Object.entries(updates)) {
    if (value === undefined) continue;
    if (snapshot.dynamic.includes(name))
      throw new NayutaError('errorConfig', { field: name });
    const matches = object.properties.filter(
      (property) => property.name && propertyName(property.name) === name,
    );
    if (matches.length > 1)
      throw new NayutaError('errorConfig', { field: name });
    const property = matches[0];
    const merged =
      name === 'copyright'
        ? {
            ...snapshot.values.copyright,
            ...Object.fromEntries(
              Object.entries(value as object).filter(
                ([, item]) => item !== undefined,
              ),
            ),
          }
        : value;
    const text = JSON.stringify(merged);
    if (property && ts.isPropertyAssignment(property))
      edits.push({
        start: property.initializer.getStart(source),
        end: property.initializer.getEnd(),
        text,
      });
    else if (property) throw new NayutaError('errorConfig', { field: name });
    else additions.push(`${name}: ${text}`);
  }
  if (additions.length) {
    const last = object.properties.at(-1);
    const newline = snapshot.source.includes('\r\n') ? '\r\n' : '\n';
    if (last && !object.properties.hasTrailingComma)
      edits.push({ start: last.getEnd(), end: last.getEnd(), text: ',' });
    edits.push({
      start: object.getEnd() - 1,
      end: object.getEnd() - 1,
      text: `  ${additions.join(`,${newline}  `)},${newline}`,
    });
  }
  let output = snapshot.source;
  for (const edit of edits.sort((a, b) => b.start - a.start))
    output = output.slice(0, edit.start) + edit.text + output.slice(edit.end);
  return output;
}
export async function writeSiteConfig(
  snapshot: SiteConfigSnapshot,
  content: string,
): Promise<void> {
  await atomicWrite(snapshot.path, content, snapshot.source);
}
