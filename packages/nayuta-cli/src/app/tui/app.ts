import {
  BoxRenderable,
  InputRenderable,
  InputRenderableEvents,
  ScrollBoxRenderable,
  SelectRenderable,
  SelectRenderableEvents,
  TextRenderable,
  type CliRenderer,
  type KeyEvent,
} from '@opentui/core';
import type { AppContext } from '../context';
import { describeError } from '../../core/errors';
import { locales, type MessageKey } from '../../i18n';
import { listPosts } from '../../core/post/service';
import { saveLocale } from '../../infrastructure/preferences';
import { FormView } from './components/form';
import { blogCreateForm, blogSetupForm } from './screens/blog';
import { postCreateForm } from './screens/post-create';
import { frontmatterForm } from './screens/frontmatter';
import {
  changedFields,
  type FormModel,
  type PreparedAction,
  type Screen,
} from './state';
import { theme } from './theme';

export class TuiApp {
  screen: Screen = 'home';
  busy = false;
  private body: BoxRenderable;
  private header: TextRenderable;
  private status: TextRenderable;
  private footer: TextRenderable;
  private form?: FormView;
  private model?: FormModel;
  private values: Record<string, string> = {};
  private initial: Record<string, string> = {};
  private prepared?: PreparedAction;
  private afterSave: 'home' | 'quit' = 'home';
  private previous: Screen = 'form';
  private quitAfterDiscard = false;
  private menu?: SelectRenderable;
  private search?: InputRenderable;
  private reviewScroll?: ScrollBoxRenderable;
  private keyHandler = (key: KeyEvent) => this.handleKey(key);
  constructor(
    readonly renderer: CliRenderer,
    readonly context: AppContext,
  ) {
    const root = new BoxRenderable(renderer, {
      id: 'nayuta',
      width: '100%',
      height: '100%',
      flexDirection: 'column',
      padding: 1,
      backgroundColor: theme.background,
    });
    this.header = new TextRenderable(renderer, {
      id: 'header',
      content: '',
      fg: theme.accent,
      flexShrink: 0,
    });
    this.status = new TextRenderable(renderer, {
      id: 'status',
      content: '',
      fg: theme.error,
      flexShrink: 0,
    });
    this.body = new BoxRenderable(renderer, {
      id: 'body',
      flexGrow: 1,
      flexDirection: 'column',
      minHeight: 1,
    });
    this.footer = new TextRenderable(renderer, {
      id: 'footer',
      content: '',
      fg: theme.muted,
      flexShrink: 0,
    });
    root.add(this.header);
    root.add(this.status);
    root.add(this.body);
    root.add(this.footer);
    renderer.root.add(root);
    renderer.keyInput.on('keypress', this.keyHandler);
  }
  start(): void {
    this.home();
  }
  private t(key: MessageKey): string {
    return this.context.i18n.t(key);
  }
  private clear(
    screen: Screen,
    title: MessageKey,
    hint: MessageKey = 'footer',
  ): void {
    for (const child of this.body.getChildren()) child.destroyRecursively();
    this.form = undefined;
    this.menu = undefined;
    this.search = undefined;
    this.reviewScroll = undefined;
    this.screen = screen;
    this.header.content = `Nayuta · ${locales.find((locale) => locale.id === this.context.i18n.locale)!.name}\n${this.t(title)} · ${this.context.cwd}\n`;
    this.status.content = '';
    this.footer.content = this.t(hint);
  }
  private choose(
    items: { name: string; value: string; description?: string }[],
    action: (value: string) => void,
  ): SelectRenderable {
    const select = new SelectRenderable(this.renderer, {
      id: 'menu',
      flexGrow: 1,
      options: items.map((item) => ({
        ...item,
        description: item.description ?? '',
      })),
      showDescription: false,
      backgroundColor: theme.background,
      textColor: theme.text,
      selectedBackgroundColor: theme.selected,
      wrapSelection: true,
    });
    select.on(
      SelectRenderableEvents.ITEM_SELECTED,
      (_index: number, option: { value: string }) => action(option.value),
    );
    this.body.add(select);
    this.menu = select;
    select.focus();
    return select;
  }
  home(): void {
    this.model = undefined;
    this.prepared = undefined;
    this.afterSave = 'home';
    this.clear('home', 'appDescription');
    const keys: MessageKey[] = [
      'createBlog',
      'setupBlog',
      'createPost',
      'editFrontmatter',
      'settings',
      'quit',
    ];
    this.choose(
      keys.map((key) => ({ name: this.t(key), value: key })),
      (value) => {
        void this.open(value).catch((error) => this.showError(error));
      },
    );
  }
  async open(action: string): Promise<void> {
    if (action === 'quit') {
      this.destroy();
      return;
    }
    if (action === 'settings') {
      this.settings();
      return;
    }
    if (action === 'editFrontmatter') {
      await this.posts();
      return;
    }
    const model =
      action === 'createBlog'
        ? await blogCreateForm(this.context)
        : action === 'setupBlog'
          ? await blogSetupForm(this.context)
          : postCreateForm(this.context);
    this.openForm(model);
  }
  openForm(model: FormModel): void {
    this.model = model;
    this.initial = Object.fromEntries(
      model.fields.map((field) => [field.name, field.value]),
    );
    this.values = { ...this.initial };
    this.renderForm();
  }
  private renderForm(): void {
    this.clear('form', this.model!.title, 'formHelp');
    this.form = new FormView(
      this.renderer,
      this.model!.fields,
      this.context.i18n,
      this.values,
    );
    this.body.add(this.form.root);
    this.form.focus();
  }
  private collect(): void {
    if (this.form && this.model)
      this.values = this.form.values(this.model.fields);
  }
  private dirty(): boolean {
    this.collect();
    return Boolean(
      this.model && changedFields(this.initial, this.values).length,
    );
  }
  async review(): Promise<void> {
    if (this.busy) return;
    this.collect();
    this.busy = true;
    this.status.content = this.t('working');
    try {
      this.prepared = await this.model!.prepare(this.values);
      this.renderReview();
    } finally {
      this.busy = false;
    }
  }
  private renderReview(): void {
    this.clear('review', 'changes', 'diffHelp');
    const scroll = new ScrollBoxRenderable(this.renderer, {
      id: 'review-scroll',
      flexGrow: 1,
      scrollY: true,
      scrollX: true,
    });
    scroll.add(
      new TextRenderable(this.renderer, {
        id: 'diff',
        content: this.prepared!.preview,
        fg: theme.text,
        flexShrink: 0,
      }),
    );
    this.body.add(scroll);
    this.reviewScroll = scroll;
    scroll.focus();
    const menu = this.choose(
      [
        { name: this.t('save'), value: 'save' },
        { name: this.t('back'), value: 'back' },
      ],
      (value) => {
        if (value === 'save') void this.save();
        else this.renderForm();
      },
    );
    menu.height = 2;
    menu.flexGrow = 0;
  }
  async save(): Promise<void> {
    if (this.busy || !this.prepared) return;
    this.busy = true;
    this.status.content = this.t('working');
    try {
      const result = await this.prepared.save();
      const created =
        this.model?.title === 'createBlog' ||
        this.model?.title === 'createPost';
      if (this.afterSave === 'quit') this.destroy();
      else {
        this.home();
        this.status.content = this.context.i18n.t(
          created ? 'createdAt' : 'updatedAt',
          {
            path: result.path,
          },
        );
      }
    } catch (error) {
      this.showError(error);
    } finally {
      this.busy = false;
    }
  }
  private requestLeave(quit: boolean): void {
    if (!this.dirty()) {
      if (quit) this.destroy();
      else this.home();
      return;
    }
    this.previous = this.screen;
    this.quitAfterDiscard = quit;
    this.clear('confirm', 'unsaved');
    this.choose(
      ['cancel', 'discard', 'save'].map((key) => ({
        name: this.t(key as MessageKey),
        value: key,
      })),
      (value) => {
        if (value === 'cancel') {
          if (this.previous === 'review') this.renderReview();
          else this.renderForm();
        } else if (value === 'discard') {
          if (this.quitAfterDiscard) this.destroy();
          else this.home();
        } else {
          this.afterSave = this.quitAfterDiscard ? 'quit' : 'home';
          void this.review().catch((error) => {
            this.renderForm();
            this.showError(error);
          });
        }
      },
    );
  }
  private async posts(): Promise<void> {
    const posts = await listPosts(this.context.cwd);
    this.clear('posts', 'editFrontmatter', 'postsHelp');
    const search = new InputRenderable(this.renderer, {
      id: 'post-search',
      placeholder: this.t('search'),
      backgroundColor: theme.background,
      textColor: theme.text,
    });
    this.body.add(search);
    this.search = search;
    const select = this.choose([], (path) => {
      void frontmatterForm(this.context, path)
        .then((model) => this.openForm(model))
        .catch((error) => this.showError(error));
    });
    const filter = () => {
      const query = search.value.toLocaleLowerCase();
      select.options = posts
        .filter((post) =>
          `${post.title} ${post.id}`.toLocaleLowerCase().includes(query),
        )
        .map((post) => ({
          name: `${post.draft ? `[${this.t('draft')}] ` : ''}${post.title} · ${post.publishDate}`,
          value: post.path,
          description: post.id,
        }));
      this.status.content = select.options.length ? '' : this.t('noPosts');
    };
    search.on(InputRenderableEvents.INPUT, filter);
    search.on(InputRenderableEvents.ENTER, () => select.focus());
    filter();
    search.focus();
  }
  private settings(): void {
    this.clear('settings', 'language');
    this.choose(
      locales.map((locale) => ({ name: locale.name, value: locale.id })),
      (value) => {
        void saveLocale(value as typeof this.context.i18n.locale)
          .then(() => {
            this.context.i18n.locale = value as typeof this.context.i18n.locale;
            this.home();
          })
          .catch((error) => this.showError(error));
      },
    );
  }
  private showError(error: unknown): void {
    this.status.content = describeError(error, this.context.i18n);
  }
  private handleKey(key: KeyEvent): void {
    if (this.busy) {
      key.preventDefault();
      return;
    }
    if (key.ctrl && key.name === 'c') {
      key.preventDefault();
      this.requestLeave(true);
    } else if (key.name === 'escape') {
      key.preventDefault();
      if (this.screen === 'review') this.renderForm();
      else if (this.screen === 'confirm') {
        if (this.previous === 'review') this.renderReview();
        else this.renderForm();
      } else this.requestLeave(false);
    } else if (key.name === 'tab') {
      key.preventDefault();
      if (this.form) this.form.move(key.shift ? -1 : 1);
      else if (this.screen === 'posts') {
        if (this.search?.focused) this.menu?.focus();
        else this.search?.focus();
      } else if (this.screen === 'review') {
        if (this.menu?.focused) this.reviewScroll?.focus();
        else this.menu?.focus();
      }
    } else if (key.ctrl && key.name === 's') {
      key.preventDefault();
      if (this.screen === 'form')
        void this.review().catch((error) => this.showError(error));
      else if (this.screen === 'review') void this.save();
    } else if (
      this.screen === 'posts' &&
      key.name === 'down' &&
      this.search?.focused
    ) {
      key.preventDefault();
      this.menu?.focus();
    }
  }
  destroy(): void {
    this.renderer.keyInput.off('keypress', this.keyHandler);
    this.renderer.destroy();
  }
}
