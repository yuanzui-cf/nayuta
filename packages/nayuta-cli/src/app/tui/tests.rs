use super::*;
use crate::i18n::I18n;
fn key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.key(KeyEvent::new(code, modifiers));
}
fn edit(app: &mut App, path: &str, text: &str) {
    let Page::Form { form, .. } = &mut app.page else {
        panic!("form expected")
    };
    form.focus = form.fields().iter().position(|f| f.path == path).unwrap();
    key(app, KeyCode::Enter, KeyModifiers::NONE);
    key(app, KeyCode::Char('k'), KeyModifiers::CONTROL);
    for c in text.chars() {
        key(app, KeyCode::Char(c), KeyModifiers::NONE);
    }
    key(app, KeyCode::Enter, KeyModifiers::NONE);
    assert!(app.editing.is_none(), "{:?}", app.message);
}
fn wait(app: &mut App) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while app.job.is_some() && std::time::Instant::now() < deadline {
        app.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(app.job.is_none());
    assert!(app.message.is_none(), "{:?}", app.message);
}
#[test]
fn creation_settings_article_and_frontmatter_use_the_same_forms() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let folder = tempfile::tempdir().unwrap();
    let cwd = folder.path().canonicalize().unwrap();
    crate::infra::git::run(&cwd, &["init", "--initial-branch=main"]).unwrap();
    crate::infra::git::run(&cwd, &["config", "user.name", "UI Fixture"]).unwrap();
    crate::infra::git::run(&cwd, &["config", "user.email", "ui@example.org"]).unwrap();
    let mut app = App::new(Ui {
        cwd,
        project: None,
        i18n: I18n::default(),
        color: false,
        mouse: false,
    });
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(Output::Template(Template {
        root: source,
        reference: "test".into(),
        commit: "0000000000000000000000000000000000000000".into(),
    })))
    .unwrap();
    app.job = Some(Job {
        rx,
        cancelled: false,
        writes: false,
    });
    app.poll();
    edit(&mut app, "title", "我的博客");
    edit(&mut app, "site_url", "https://example.org");
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    wait(&mut app);
    assert!(matches!(app.page, Page::Success(_)));
    let root = app.ui.project.as_ref().unwrap().root.clone();
    assert!(root.join("src/content/assets/logo.svg").is_file() && !root.join("public").exists());
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    edit(&mut app, "description", "界面保存");
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    wait(&mut app);
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    edit(&mut app, "_slug", "hello");
    edit(&mut app, "title", "你好，Nayuta");
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    wait(&mut app);
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    key(&mut app, KeyCode::Down, KeyModifiers::NONE);
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    wait(&mut app);
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    edit(&mut app, "description", "文章资料保存");
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    wait(&mut app);
    let project = Project::open(&root).unwrap();
    assert_eq!(project.config().unwrap()["description"], "界面保存");
    let file = content::paths(&project.schema.posts_root)
        .unwrap()
        .pop()
        .unwrap();
    let article = Content::open(&file).unwrap();
    assert_eq!(article.values["title"], "你好，Nayuta");
    assert_eq!(article.values["description"], "文章资料保存");
    assert_eq!(article.values["draft"], true);
    assert_eq!(
        crate::infra::git::run(&root, &["rev-list", "--all", "--count"]).unwrap(),
        "1"
    );
}
