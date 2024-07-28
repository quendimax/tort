use std::ops::Range;
use std::sync::Arc;


pub type Source<'source> = Arc<str>;
pub type SourceRange = Range<usize>;

