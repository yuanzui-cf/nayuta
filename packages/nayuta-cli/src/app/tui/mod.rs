mod input;
mod terminal;
#[cfg(test)]
mod tests;
mod view;

use crate::{
    app::cli::Ui,
    core::{
        context::Project,
        create,
        form::Form,
        frontmatter::Frontmatter,
        post::{self, NewPost},
        schema::{Field, Kind, Schema},
        settings::Settings,
    },
    infra::{
        content::{self, Content},
        error::{Result, user},
        git::{
            github,
            template::{self, Template},
        },
    },
};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};
use input::Input;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{
        atomic::Ordering,
        mpsc::{self, Receiver},
    },
    time::Duration,
};

#[derive(Clone)]
enum Editor {
    Create(Template),
    Settings(Project, Settings),
    Post(Project),
    Frontmatter(Project, Frontmatter),
}

enum Page {
    Menu(usize),
    Versions {
        selected: usize,
        submit: bool,
    },
    Reference(Input),
    Form {
        editor: Box<Editor>,
        form: Box<Form>,
    },
    Posts {
        entries: Vec<(PathBuf, String)>,
        filter: Input,
        selected: usize,
    },
    Success(String),
}
enum Output {
    Versions(Vec<String>),
    Template(Template),
    Created(Box<Project>),
    Saved(String),
    Posts(Vec<(PathBuf, String)>),
}
struct Job {
    rx: Receiver<Result<Output>>,
    cancelled: bool,
    writes: bool,
}
struct Editing {
    field: crate::core::form::Field,
    input: Input,
}

struct App {
    ui: Ui,
    page: Page,
    versions: Vec<String>,
    job: Option<Job>,
    editing: Option<Editing>,
    message: Option<String>,
    discard: bool,
    quit: Option<u8>,
    hits: Vec<(Rect, usize)>,
}

pub fn run(ui: Ui) -> Result<u8> {
    let _guard = terminal::Guard::enter(ui.mouse)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut app = App::new(ui);
    if app.ui.project.is_none() {
        app.load_versions();
    }
    loop {
        app.poll();
        if terminal::INTERRUPTED.swap(false, Ordering::SeqCst) {
            app.interrupt();
        }
        if let Some(code) = app.quit
            && app.job.is_none()
        {
            return Ok(code);
        }
        terminal.draw(|frame| view::draw(frame, &mut app))?;
        if event::poll(Duration::from_millis(80))? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => app.key(key),
                Event::Paste(text) => {
                    if let Some(edit) = &mut app.editing {
                        edit.input.insert(&text);
                    } else if let Page::Reference(input) = &mut app.page {
                        input.insert(&text);
                    } else if let Page::Posts { filter, .. } = &mut app.page {
                        filter.insert(&text);
                    }
                }
                Event::Mouse(mouse)
                    if app.ui.mouse && app.editing.is_none() && app.job.is_none() =>
                {
                    match mouse.kind {
                        MouseEventKind::Down(event::MouseButton::Left) => {
                            if let Some((_, focus)) = app
                                .hits
                                .iter()
                                .find(|(r, _)| r.contains((mouse.column, mouse.row).into()))
                            {
                                app.focus(*focus);
                            }
                        }
                        MouseEventKind::ScrollDown => app.move_focus(false),
                        MouseEventKind::ScrollUp => app.move_focus(true),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

impl App {
    fn new(ui: Ui) -> Self {
        let page = if ui.project.is_some() {
            Page::Menu(0)
        } else {
            Page::Versions {
                selected: 0,
                submit: false,
            }
        };
        Self {
            ui,
            page,
            versions: Vec::new(),
            job: None,
            editing: None,
            message: None,
            discard: false,
            quit: None,
            hits: Vec::new(),
        }
    }
    fn spawn(&mut self, writes: bool, task: impl FnOnce() -> Result<Output> + Send + 'static) {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = task();
            let _ = tx.send(result);
        });
        self.job = Some(Job {
            rx,
            cancelled: false,
            writes,
        });
        self.message = None;
    }
    fn load_versions(&mut self) {
        self.spawn(false, || {
            Ok(Output::Versions(
                github::releases(false)?
                    .into_iter()
                    .map(|r| r.tag_name)
                    .collect(),
            ))
        });
    }
    fn interrupt(&mut self) {
        self.quit = Some(130);
        if let Some(job) = &mut self.job {
            job.cancelled = true;
        }
    }
    fn poll(&mut self) {
        let Some(job) = &self.job else {
            return;
        };
        let result = match job.rx.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(_) => Err(user("error", "worker stopped")),
        };
        let job = self.job.take().unwrap();
        if job.cancelled && !job.writes {
            return;
        }
        match result {
            Err(error) => {
                match &mut self.page {
                    Page::Versions { submit, .. } => *submit = false,
                    Page::Reference(_) => {
                        self.page = Page::Versions {
                            selected: self.versions.len(),
                            submit: false,
                        }
                    }
                    _ => {}
                }
                self.message = Some(self.ui.i18n.error(&error));
            }
            Ok(Output::Versions(versions)) => {
                self.versions = versions;
                self.page = Page::Versions {
                    selected: 0,
                    submit: false,
                };
            }
            Ok(Output::Template(template)) => {
                match crate::infra::config::schema::load(&template.root).and_then(|schema| {
                    create::defaults(&template, &self.ui.cwd).map(|values| (schema, values))
                }) {
                    Ok((schema, mut values)) => {
                        values["_directory"] = json!("my-blog");
                        let schema = with_meta(
                            schema.config,
                            vec![("_directory", Schema::new(Kind::String))],
                        );
                        self.page = Page::Form {
                            editor: Box::new(Editor::Create(template)),
                            form: Box::new(Form::new(schema, values)),
                        };
                    }
                    Err(error) => self.message = Some(self.ui.i18n.error(&error)),
                }
            }
            Ok(Output::Created(project)) => {
                let path = project.root.display().to_string();
                self.ui.project = Some(*project);
                self.page = Page::Success(format!(
                    "{}: {path}\n{}",
                    self.ui.i18n.t("created"),
                    self.ui.i18n.t("next_steps")
                ));
            }
            Ok(Output::Saved(path)) => {
                self.page = Page::Success(format!("{}\n{path}", self.ui.i18n.t("done")))
            }
            Ok(Output::Posts(entries)) => {
                self.page = Page::Posts {
                    entries,
                    filter: Input::new(String::new()),
                    selected: 0,
                }
            }
        }
    }
    fn focus(&mut self, focus: usize) {
        match &mut self.page {
            Page::Menu(selected) => *selected = focus.min(2),
            Page::Versions { selected, submit } => {
                *submit = focus > self.versions.len();
                if !*submit {
                    *selected = focus.min(self.versions.len());
                }
            }
            Page::Form { form, .. } => form.focus = focus.min(form.fields().len()),
            Page::Posts { selected, .. } => *selected = focus,
            _ => {}
        }
    }
    fn move_focus(&mut self, reverse: bool) {
        let cycle = |i: usize, n: usize| {
            if reverse {
                (i + n - 1) % n
            } else {
                (i + 1) % n
            }
        };
        match &mut self.page {
            Page::Menu(selected) => *selected = cycle(*selected, 3),
            Page::Versions { selected, submit } if !*submit => {
                *selected = cycle(*selected, self.versions.len() + 1);
            }
            Page::Form { form, .. } => {
                form.focus = cycle(form.focus.min(form.fields().len()), form.fields().len() + 1)
            }
            Page::Posts {
                entries,
                filter,
                selected,
            } => {
                let n = filtered(entries, &filter.text).len();
                if n > 0 {
                    *selected = cycle((*selected).min(n - 1), n);
                }
            }
            _ => {}
        }
    }
    fn key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if key.code == KeyCode::Char('c') && ctrl {
            self.interrupt();
            return;
        }
        if let Some(job) = &mut self.job {
            if key.code == KeyCode::Esc && !job.writes {
                job.cancelled = true;
            }
            return;
        }
        if self.discard {
            if key.code == KeyCode::Enter {
                self.discard = false;
                self.back();
            } else if key.code == KeyCode::Esc {
                self.discard = false;
            }
            return;
        }
        if self.editing.is_some() {
            match key.code {
                KeyCode::Esc => self.editing = None,
                KeyCode::Enter => {
                    let editing = self.editing.take().unwrap();
                    if let Page::Form { form, .. } = &mut self.page
                        && let Err(error) = form.apply(&editing.field, &editing.input.text)
                    {
                        self.message = Some(self.ui.i18n.error(&error));
                        self.editing = Some(editing);
                    }
                }
                _ => self.editing.as_mut().unwrap().input.key(key),
            }
            return;
        }
        if key.code == KeyCode::Esc {
            if let Page::Form { form, .. } = &self.page
                && form.dirty()
            {
                self.discard = true;
            } else {
                self.back();
            }
            return;
        }
        if ctrl && key.code == KeyCode::Char('s') {
            self.submit();
            return;
        }
        self.message = None;
        match key.code {
            KeyCode::Tab | KeyCode::BackTab => {
                if let Page::Versions { submit, .. } = &mut self.page {
                    *submit = !*submit;
                } else {
                    self.move_focus(
                        key.code == KeyCode::BackTab || key.modifiers.contains(KeyModifiers::SHIFT),
                    );
                }
            }
            KeyCode::Up => self.move_focus(true),
            KeyCode::Down => self.move_focus(false),
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.move_focus(true);
                }
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.move_focus(false);
                }
            }
            KeyCode::Enter => self.activate(false),
            KeyCode::Char(' ') if matches!(self.page, Page::Form { .. }) => self.activate(false),
            KeyCode::Left | KeyCode::Right if matches!(self.page, Page::Form { .. }) => {
                self.activate(key.code == KeyCode::Left)
            }
            KeyCode::Char('u') if ctrl => {
                if let Page::Form { form, .. } = &mut self.page
                    && let Some(field) = form.fields().get(form.focus)
                    && let Err(error) = form.unset(field)
                {
                    self.message = Some(self.ui.i18n.error(&error));
                }
            }
            KeyCode::Char('r') if matches!(self.page, Page::Versions { .. }) => {
                self.load_versions()
            }
            _ => match &mut self.page {
                Page::Reference(input) => input.key(key),
                Page::Posts {
                    filter, selected, ..
                } => {
                    filter.key(key);
                    *selected = 0;
                }
                _ => {}
            },
        }
    }
    fn back(&mut self) {
        self.message = None;
        self.editing = None;
        match self.page {
            Page::Menu(_) | Page::Versions { .. } | Page::Success(_) => self.quit = Some(0),
            Page::Reference(_) => {
                self.page = Page::Versions {
                    selected: self.versions.len(),
                    submit: false,
                }
            }
            Page::Form { ref editor, .. } if matches!(**editor, Editor::Create(_)) => {
                self.page = Page::Versions {
                    selected: 0,
                    submit: false,
                }
            }
            _ => self.page = Page::Menu(0),
        }
    }
    fn activate(&mut self, reverse: bool) {
        match &mut self.page {
            Page::Form { form, .. } => {
                let Some(field) = form.fields().get(form.focus).cloned() else {
                    self.submit();
                    return;
                };
                match form.cycle(&field, reverse) {
                    Ok(true) => form.focus = form.focus.min(form.fields().len()),
                    Err(error) => self.message = Some(self.ui.i18n.error(&error)),
                    Ok(false) => {
                        let value = crate::core::schema::get(&form.values, &field.path);
                        let text = value
                            .map(|v| {
                                v.as_str()
                                    .filter(|_| field.schema.text_input())
                                    .map(str::to_owned)
                                    .unwrap_or_else(|| v.to_string())
                            })
                            .unwrap_or_default();
                        self.editing = Some(Editing {
                            field,
                            input: Input::new(text),
                        });
                    }
                }
            }
            Page::Versions { selected, submit } => {
                if !*submit {
                    *submit = true;
                    return;
                }
                if *selected == self.versions.len() {
                    self.page = Page::Reference(Input::new(String::new()));
                } else {
                    let reference = self.versions[*selected].clone();
                    self.fetch(reference);
                }
            }
            Page::Reference(input) => {
                let reference = input.text.trim().to_owned();
                self.fetch(reference);
            }
            Page::Menu(selected) => {
                let project = self.ui.project.as_ref().unwrap().clone();
                match selected {
                    0 => match Settings::open(&project) {
                        Ok(settings) => {
                            let mut form =
                                Form::new(settings.schema.clone(), settings.values.clone());
                            form.readonly = settings.readonly.clone();
                            self.page = Page::Form {
                                editor: Box::new(Editor::Settings(project, settings)),
                                form: Box::new(form),
                            };
                        }
                        Err(error) => self.message = Some(self.ui.i18n.error(&error)),
                    },
                    1 => {
                        let mut values = post::defaults(&project);
                        values["_slug"] = json!("");
                        values["_format"] = json!("md");
                        values["_date_prefix"] = json!(true);
                        let schema = with_meta(
                            project.schema.post.clone(),
                            vec![
                                ("_slug", Schema::new(Kind::String)),
                                (
                                    "_format",
                                    Schema::new(Kind::Union(vec![
                                        Schema::new(Kind::Literal(json!("md"))),
                                        Schema::new(Kind::Literal(json!("mdx"))),
                                    ])),
                                ),
                                ("_date_prefix", Schema::new(Kind::Boolean)),
                            ],
                        );
                        let mut form = Form::new(schema, values);
                        form.inherited = crate::core::context::static_config(&project.root)
                            .map(|(values, _)| values)
                            .unwrap_or_else(|_| json!({}));
                        self.page = Page::Form {
                            editor: Box::new(Editor::Post(project)),
                            form: Box::new(form),
                        };
                    }
                    _ => self.spawn(false, move || {
                        let entries = content::paths(&project.schema.posts_root)?
                            .into_iter()
                            .map(|path| {
                                let title = Content::open(&path)
                                    .map(|c| {
                                        c.values["title"].as_str().unwrap_or_default().to_owned()
                                    })
                                    .unwrap_or_default();
                                Ok((path, title))
                            })
                            .collect::<Result<_>>()?;
                        Ok(Output::Posts(entries))
                    }),
                }
            }
            Page::Posts {
                entries,
                filter,
                selected,
            } => {
                let items = filtered(entries, &filter.text);
                if let Some((path, _)) = items.get(*selected) {
                    let project = self.ui.project.as_ref().unwrap().clone();
                    match Frontmatter::open(&project, path) {
                        Ok(editor) => {
                            let mut form =
                                Form::new(editor.schema.clone(), editor.content.values.clone());
                            form.inherited = crate::core::context::static_config(&project.root)
                                .map(|(values, _)| values)
                                .unwrap_or_else(|_| json!({}));
                            self.page = Page::Form {
                                editor: Box::new(Editor::Frontmatter(project, editor)),
                                form: Box::new(form),
                            };
                        }
                        Err(error) => self.message = Some(self.ui.i18n.error(&error)),
                    }
                }
            }
            Page::Success(_) => self.page = Page::Menu(0),
        }
    }
    fn fetch(&mut self, reference: String) {
        self.spawn(false, move || {
            Ok(Output::Template(template::fetch(&reference, false)?))
        });
    }
    fn submit(&mut self) {
        let Page::Form { editor, form } = &self.page else {
            self.activate(false);
            return;
        };
        let editor = (**editor).clone();
        let mut values = form.values.clone();
        let cwd = self.ui.cwd.clone();
        self.spawn(true, move || match editor {
            Editor::Create(template) => {
                let directory = values
                    .as_object_mut()
                    .unwrap()
                    .remove("_directory")
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .filter(|s| !s.trim().is_empty())
                    .ok_or_else(|| user("required", "directory"))?;
                Ok(Output::Created(Box::new(create::create(
                    &template,
                    &PathBuf::from(directory),
                    &mut values,
                    &cwd,
                )?)))
            }
            Editor::Settings(project, editor) => {
                editor.save(&project, &mut values)?;
                Ok(Output::Saved(
                    project.root.join("src/config.ts").display().to_string(),
                ))
            }
            Editor::Post(project) => {
                let map = values.as_object_mut().unwrap();
                let slug = map
                    .remove("_slug")
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default();
                let format = map
                    .remove("_format")
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "md".into());
                let date_prefix = map
                    .remove("_date_prefix")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let path = post::create(
                    &project,
                    &mut NewPost {
                        slug,
                        format,
                        date_prefix,
                        values,
                    },
                )?;
                Ok(Output::Saved(path.display().to_string()))
            }
            Editor::Frontmatter(project, editor) => {
                editor.save(&project, &mut values)?;
                Ok(Output::Saved(editor.content.path.display().to_string()))
            }
        });
    }
}

fn with_meta(mut schema: Schema, fields: Vec<(&str, Schema)>) -> Schema {
    if let Kind::Object(list) = &mut schema.kind {
        for (name, schema) in fields.into_iter().rev() {
            list.insert(
                0,
                Field {
                    name: name.into(),
                    schema,
                },
            );
        }
    }
    schema
}
fn filtered<'a>(entries: &'a [(PathBuf, String)], filter: &str) -> Vec<&'a (PathBuf, String)> {
    let filter = filter.to_lowercase();
    entries
        .iter()
        .filter(|(path, title)| {
            title.to_lowercase().contains(&filter)
                || path.to_string_lossy().to_lowercase().contains(&filter)
        })
        .collect()
}
