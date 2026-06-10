use rust_embed::{EmbeddedFile, RustEmbed};

#[derive(RustEmbed)]
#[folder = "src/dist"]
struct Asset;

pub fn get_embedded_file(path: &str) -> Option<EmbeddedFile> {
    Asset::get(path)
}
