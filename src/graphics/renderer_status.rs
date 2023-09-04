#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum RendererStatus {
    #[default]
    Empty,
    NeedsUpload,
    Ready,
}
