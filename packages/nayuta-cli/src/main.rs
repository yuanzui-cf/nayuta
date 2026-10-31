use nayuta_cli::app::{
    cli::{self, Launch},
    tui,
};

fn main() {
    let code = match cli::launch(std::env::args_os().collect()) {
        Launch::Exit(code) => code,
        Launch::Tui(ui) => {
            let i18n = ui.i18n.clone();
            match tui::run(*ui) {
                Ok(code) => code,
                Err(error) => {
                    eprintln!("{}", i18n.error(&error));
                    1
                }
            }
        }
    };
    nayuta_cli::infra::logging::shutdown();
    std::process::exit(i32::from(code));
}
