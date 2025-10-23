pub mod fs;
pub mod infer;
pub mod rollup;
pub mod sampler;
pub mod store;
pub mod trigger;
pub mod types;
pub mod util;

use crate::config::Config;
use anyhow::Result;

use self::store::{create_state, Store};

pub use infer::maybe_update_current;

#[allow(unused_imports)]
pub use types::Persona;
pub use types::{PersonaSource, PersonaState};

pub fn init(_config: &Config) -> Result<PersonaState> {
    let store = Store::new()?;

    if let Some(persona) = store.read_override()? {
        let state = create_state(persona, 1.0, PersonaSource::Override);
        store.write_current(&state)?;
        return Ok(state);
    }

    if let Some(state) = store.read_current()? {
        return Ok(state);
    }

    store.ensure_current_exists()
}

pub fn start_background_tasks(cfg: &Config) -> Result<()> {
    if !cfg.persona.enabled {
        return Ok(());
    }

    fs::ensure_dirs()?;
    rollup::catch_up()?;
    sampler::spawn(cfg)?;
    infer::schedule_daily(cfg.clone())?;
    trigger::spawn(cfg.clone())?;
    Ok(())
}
