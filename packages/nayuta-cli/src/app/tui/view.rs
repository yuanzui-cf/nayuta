use super::{App, Editor, Page, filtered};
use crate::core::schema::get;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

fn focus(app: &App) -> Style {
    if app.ui.color {
        Style::default()
            .fg(Color::LightBlue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
    }
}
fn quiet(app: &App) -> Style {
    if app.ui.color {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    }
}
pub(super) fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(2));
    let height = height.min(area.height.saturating_sub(2));
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}
fn label(app: &App, path: &str) -> String {
    let path = path.trim_start_matches('_');
    path.split('.')
        .map(|s| {
            app.ui
                .i18n
                .t(if s == "date_prefix" { "date_prefix" } else { s })
        })
        .collect::<Vec<_>>()
        .join(" · ")
}
fn button(frame: &mut Frame, app: &mut App, area: Rect, text: String, active: bool, index: usize) {
    let style = if active { focus(app) } else { Style::default() };
    frame.render_widget(
        Paragraph::new(format!("{} {text}", if active { ">" } else { " " }))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if active {
                        BorderType::Double
                    } else {
                        BorderType::Plain
                    })
                    .border_style(style),
            )
            .style(style),
        area,
    );
    app.hits.push((area, index));
}
fn list(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    items: Vec<String>,
    selected: usize,
    focused: bool,
) {
    let active = if focused {
        focus(app)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    let items: Vec<_> = items
        .into_iter()
        .enumerate()
        .map(|(i, s)| ListItem::new(format!("{} {s}", if i == selected { "*" } else { " " })))
        .collect();
    let mut state = ListState::default().with_selected(Some(selected));
    frame.render_stateful_widget(
        List::new(items)
            .highlight_style(active)
            .highlight_symbol(if focused { "> " } else { "  " }),
        area,
        &mut state,
    );
    for row in 0..area.height {
        app.hits.push((
            Rect::new(area.x, area.y + row, area.width, 1),
            state.offset() + usize::from(row),
        ));
    }
}

pub(super) fn draw(frame: &mut Frame, app: &mut App) {
    app.hits.clear();
    let area = frame.area();
    if area.width < 30 || area.height < 12 {
        frame.render_widget(
            Paragraph::new(app.ui.i18n.t("terminal_small")).wrap(Wrap { trim: false }),
            area,
        );
        return;
    }
    let outer = centered(area, 92, area.height.saturating_sub(2));
    let title = match &app.page {
        Page::Menu(_) => "welcome",
        Page::Versions { .. } => "version",
        Page::Reference(_) => "ref",
        Page::Posts { .. } => "posts",
        Page::Success(_) => "done",
        Page::Form { editor, .. } => match **editor {
            Editor::Create(_) => "create",
            Editor::Settings(..) => "settings",
            Editor::Post(_) => "new_post",
            Editor::Frontmatter(..) => "frontmatter",
        },
    };
    let title = app.ui.i18n.t(title);
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" nayuta ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("· {title} ")),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.ui.color {
            Color::White
        } else {
            Color::Reset
        }));
    let inner = block.inner(outer);
    frame.render_widget(block, outer);
    let header = Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(2), 1);
    let context = format!(
        "{} · {} · {}",
        app.ui
            .project
            .as_ref()
            .map(|p| p.root.display().to_string())
            .unwrap_or_else(|| app.ui.cwd.display().to_string()),
        app.ui
            .project
            .as_ref()
            .map(|p| p.version.as_str())
            .unwrap_or(env!("CARGO_PKG_VERSION")),
        app.ui.i18n.locale.name()
    );
    frame.render_widget(Paragraph::new(context).style(quiet(app)), header);
    let footer = Rect::new(
        inner.x + 1,
        inner.bottom().saturating_sub(3),
        inner.width.saturating_sub(2),
        3,
    );
    let body = Rect::new(
        inner.x + 1,
        inner.y + 2,
        inner.width.saturating_sub(2),
        inner.height.saturating_sub(6),
    );
    let mut help = app.ui.i18n.t("help_keys");
    // Move the page out temporarily to allow widget hit maps and focus styles to share App.
    let mut page = std::mem::replace(&mut app.page, Page::Menu(0));
    match &mut page {
        Page::Menu(selected) => {
            let box_area = centered(body, 60, 8);
            frame.render_widget(
                Paragraph::new(app.ui.i18n.t("welcome")).alignment(Alignment::Center),
                Rect::new(box_area.x, box_area.y, box_area.width, 1),
            );
            list(
                frame,
                app,
                Rect::new(box_area.x, box_area.y + 3, box_area.width, 3),
                ["settings", "new_post", "frontmatter"]
                    .iter()
                    .map(|k| app.ui.i18n.t(k))
                    .collect(),
                *selected,
                true,
            );
        }
        Page::Versions { selected, submit } => {
            let button_area = Rect::new(body.x, body.bottom().saturating_sub(3), body.width, 3);
            let rows = Rect::new(body.x, body.y, body.width, body.height.saturating_sub(4));
            let mut versions = app.versions.clone();
            versions.push(app.ui.i18n.t("other"));
            list(frame, app, rows, versions, *selected, !*submit);
            button(
                frame,
                app,
                button_area,
                app.ui.i18n.t("continue"),
                *submit,
                app.versions.len() + 1,
            );
            help.push_str(&format!(" · {}", app.ui.i18n.t("retry_keys")));
        }
        Page::Reference(input) => {
            let box_area = centered(body, 70, 5);
            frame.render_widget(
                Paragraph::new(input.text.clone()).block(
                    Block::bordered()
                        .title(format!("> {}", app.ui.i18n.t("ref")))
                        .border_style(focus(app)),
                ),
                Rect::new(box_area.x, box_area.y, box_area.width, 3),
            );
            frame.set_cursor_position((
                box_area.x
                    + 1
                    + (UnicodeWidthStr::width(&input.text[..input.cursor]) as u16)
                        .min(box_area.width.saturating_sub(3)),
                box_area.y + 1,
            ));
            help = app.ui.i18n.t("input_keys");
        }
        Page::Form { form, .. } => {
            let fields = form.fields();
            form.focus = form.focus.min(fields.len());
            let rows = usize::from(body.height.saturating_sub(3) / 3).max(1);
            if form.focus < fields.len() {
                if form.focus < form.scroll {
                    form.scroll = form.focus;
                } else if form.focus >= form.scroll + rows {
                    form.scroll = form.focus + 1 - rows;
                }
            }
            form.scroll = form.scroll.min(fields.len().saturating_sub(rows));
            for (index, field) in fields.iter().enumerate().skip(form.scroll).take(rows) {
                let active = index == form.focus;
                let rect = Rect::new(
                    body.x,
                    body.y + ((index - form.scroll) * 3) as u16,
                    body.width,
                    3,
                );
                let mut title = format!(
                    "{} {}",
                    if active { ">" } else { " " },
                    label(app, &field.path)
                );
                if !field.schema.optional && field.schema.default.is_none() {
                    title.push_str(" *");
                }
                let value = get(&form.values, &field.path);
                let display = if field.readonly {
                    app.ui.i18n.t("read_only")
                } else if let Some(value) = value {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| value.to_string())
                } else {
                    get(&form.inherited, &field.path)
                        .or(field.schema.default.as_ref())
                        .map(|v| format!("{} · {v}", app.ui.i18n.t("inherit")))
                        .unwrap_or_else(|| app.ui.i18n.t("unset"))
                };
                let style = if active { focus(app) } else { Style::default() };
                frame.render_widget(
                    Paragraph::new(display.replace('\n', " ↵ "))
                        .style(if field.readonly {
                            quiet(app)
                        } else {
                            Style::default()
                        })
                        .block(
                            Block::bordered()
                                .title(title)
                                .border_style(style)
                                .border_type(if active {
                                    BorderType::Double
                                } else {
                                    BorderType::Plain
                                }),
                        ),
                    rect,
                );
                app.hits.push((rect, index));
            }
            let position = format!(
                "{}–{} / {}",
                form.scroll + 1,
                (form.scroll + rows).min(fields.len()),
                fields.len()
            );
            let save = Rect::new(body.x, body.bottom().saturating_sub(3), body.width, 3);
            button(
                frame,
                app,
                save,
                format!(
                    "{}{} · {position}",
                    app.ui.i18n.t("save"),
                    if form.dirty() { " *" } else { "" }
                ),
                form.focus == fields.len(),
                fields.len(),
            );
            help.push('\n');
            help.push_str(&app.ui.i18n.t("edit_keys"));
        }
        Page::Posts {
            entries,
            filter,
            selected,
        } => {
            frame.render_widget(
                Paragraph::new(format!("{}: {}", app.ui.i18n.t("filter"), filter.text))
                    .style(focus(app)),
                Rect::new(body.x, body.y, body.width, 1),
            );
            let entries = filtered(entries, &filter.text);
            *selected = (*selected).min(entries.len().saturating_sub(1));
            let items = entries
                .iter()
                .map(|(path, title)| {
                    format!(
                        "{title} · {}",
                        path.strip_prefix(&app.ui.project.as_ref().unwrap().schema.posts_root)
                            .unwrap_or(path)
                            .display()
                    )
                })
                .collect();
            list(
                frame,
                app,
                Rect::new(
                    body.x,
                    body.y + 2,
                    body.width,
                    body.height.saturating_sub(2),
                ),
                items,
                *selected,
                true,
            );
        }
        Page::Success(text) => {
            frame.render_widget(
                Paragraph::new(text.clone())
                    .wrap(Wrap { trim: false })
                    .alignment(Alignment::Center),
                centered(body, 76, 6),
            );
            help = app.ui.i18n.t("success_keys");
        }
    }
    app.page = page;
    if let Some(message) = &app.message {
        help = format!("{message}\n{help}");
    }
    frame.render_widget(
        Paragraph::new(help).wrap(Wrap { trim: false }).style(
            if app.message.is_some() && app.ui.color {
                Style::default().fg(Color::LightRed)
            } else {
                quiet(app)
            },
        ),
        footer,
    );
    if let Some(edit) = &app.editing {
        let popup = centered(area, 78, 7);
        frame.render_widget(Clear, popup);
        let block = Block::bordered()
            .title(format!("> {}", label(app, &edit.field.path)))
            .border_type(BorderType::Double)
            .border_style(focus(app));
        let inner = block.inner(popup);
        frame.render_widget(block, popup);
        // Keep the active grapheme visible even for long CJK strings.
        let mut start = 0;
        let width = usize::from(inner.width.saturating_sub(2));
        for (index, _) in edit.input.text[..edit.input.cursor].grapheme_indices(true) {
            if UnicodeWidthStr::width(&edit.input.text[index..edit.input.cursor]) <= width {
                start = index;
                break;
            }
            start = edit.input.cursor;
        }
        frame.render_widget(
            Paragraph::new(edit.input.text[start..].replace(['\n', '\r'], " ")),
            Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1),
        );
        frame.set_cursor_position((
            inner.x + 1 + UnicodeWidthStr::width(&edit.input.text[start..edit.input.cursor]) as u16,
            inner.y + 1,
        ));
        frame.render_widget(
            Paragraph::new(app.ui.i18n.t("input_keys")).wrap(Wrap { trim: false }),
            Rect::new(inner.x, inner.y + 3, inner.width, 2),
        );
    }
    if app.discard {
        popup(
            frame,
            app,
            area,
            format!(
                "{}\n\n{}",
                app.ui.i18n.t("dirty"),
                app.ui.i18n.t("confirm_discard")
            ),
        );
    }
    if let Some(job) = &app.job {
        popup(
            frame,
            app,
            area,
            format!(
                "{}\n\n{}",
                app.ui.i18n.t(if job.cancelled {
                    "cancel_pending"
                } else {
                    "progress"
                }),
                app.ui.i18n.t(if job.writes {
                    "saving_keys"
                } else {
                    "busy_keys"
                })
            ),
        );
    }
}
fn popup(frame: &mut Frame, app: &App, area: Rect, text: String) {
    let rect = centered(area, 70, 7);
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(focus(app)),
            ),
        rect,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::cli::Ui,
        i18n::{I18n, Locale},
    };
    use ratatui::{Terminal, backend::TestBackend};
    #[test]
    fn focus_is_visible_without_color_and_small_terminals_render() {
        let ui = Ui {
            cwd: std::env::temp_dir(),
            project: None,
            i18n: I18n::new(Locale::ZhTw),
            color: false,
            mouse: false,
        };
        let mut app = App::new(ui);
        app.versions = vec!["v0.3.0".into()];
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| draw(f, &mut app)).unwrap();
        let text = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(text.contains("* v0.3.0") && text.contains(">"));
        assert!(
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .any(|c| c.modifier.contains(Modifier::REVERSED))
        );
        let mut small = Terminal::new(TestBackend::new(20, 8)).unwrap();
        small.draw(|f| draw(f, &mut app)).unwrap();
    }
}
