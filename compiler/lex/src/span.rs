use std::{fmt::Display , ops::{BitOr, Range}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub inner: Range<usize>,
    // TODO: add page number? (optional; for docx inputs)
}

use chumsky::error::Span as ChumSpan;

impl ChumSpan for Span {
    type Position = usize;

    fn start(&self) -> usize { self.inner.start() }
    fn end(&self) -> usize { self.inner.end() }
    fn union(self, other: Self) -> Self { Self { inner: self.inner.union(other.inner) } }
    fn inner(self, other: Self) -> Self { Self { inner: self.inner.inner(other.inner)} }
    fn display(&self) -> Box<dyn Display + '_> { self.inner.display() }
}

impl From<Range<usize>> for Span {
    fn from(inner: Range<usize>) -> Span {
        Span { inner }
    }
}

impl Span {
    pub fn union(
        &self,
        Span {
            inner: Range { start, end },
        }: Span,
    ) -> Span {
        Span {
            inner: Range {
                start: self.inner.start.min(start),
                end: self.inner.end.max(end),
            },
        }
    }
}

impl BitOr for Span {
    type Output = Span;

    fn bitor(self, rhs: Span) -> Span { self.union(rhs) }
}
