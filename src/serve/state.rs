use crate::mirror::MirrorDirectory;

pub struct AppState {
    pub base_url: Option<String>,
    pub dir: MirrorDirectory,
}
