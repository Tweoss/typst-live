use crate::Args;
use tempdir::TempDir;
use tokio::{runtime::Runtime, sync::broadcast::Sender, sync::Notify};

pub struct ServerState {
    pub(crate) args: Args,
    pub(crate) changed: Sender<usize>,
    pub(crate) tokio: Runtime,
    pub(crate) shutdown: Notify,
    pub(crate) directory: TempDir,
}
