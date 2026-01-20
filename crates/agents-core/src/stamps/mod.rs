mod encoding;
mod types;

pub use encoding::{
    encode_comment_stamp_line, encode_stamp_meta_json, COMMENT_STAMP_PREFIX, COMMENT_STAMP_SUFFIX,
    FRONTMATTER_STAMP_KEY, JSON_STAMP_FIELD,
};
pub use types::{Stamp, StampMeta};
