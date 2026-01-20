mod apply;
mod drift;
mod encoding;
mod hash;
mod types;

pub use apply::{
    apply_stamp, parse_stamp, stamp_rendered_output, strip_existing_stamp, StampError,
};
pub use drift::{classify, DriftStatus};
pub use hash::{compute_sha256_hex, normalize_newlines};

pub use types::{Stamp, StampMeta};
