mod blocks;
mod project;
mod sprite;

type ProjectCell = std::rc::Rc<std::cell::RefCell<crate::schema::Project>>;

pub use blocks::*;
pub use project::*;
pub use sprite::*;

use std::sync::atomic::AtomicU64;
static DATA_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_next_id() -> String {
    format!(
        "ID{:08x}",
        DATA_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}
