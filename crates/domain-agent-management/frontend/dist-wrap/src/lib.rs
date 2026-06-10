use rust_embed::{Embed, EmbeddedFile};

#[derive(Embed)]
#[folder = "src/dist"]
struct Asset;

pub fn get_embedded_file(path: &str) -> Option<EmbeddedFile> {
    Asset::get(path)
}
