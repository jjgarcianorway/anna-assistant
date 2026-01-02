//! Server type definitions.

use crate::state::SharedState;

pub struct Server {
    pub(super) state: SharedState,
}
