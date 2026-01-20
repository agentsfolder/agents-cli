mod apply;
mod encoding;
mod hash;
mod types;

pub use apply::{
    apply_stamp, parse_stamp, stamp_rendered_output, strip_existing_stamp, StampError,
};
pub use encoding::{
    encode_comment_stamp_line, encode_stamp_meta_json, COMMENT_STAMP_PREFIX, COMMENT_STAMP_SUFFIX,
    FRONTMATTER_STAMP_KEY, JSON_STAMP_FIELD,
};
pub use hash::{compute_sha256_hex, normalize_newlines};
pub use types::{Stamp, StampMeta};
