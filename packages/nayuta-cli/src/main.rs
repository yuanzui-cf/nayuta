use nayuta_cli::i18n::I18n;

fn main() {
    let i18n = I18n::detect(None).unwrap_or_default();
    println!(
        "nayuta {} · {}",
        env!("CARGO_PKG_VERSION"),
        i18n.t("welcome")
    );
}
