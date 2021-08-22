use std::ops::{Deref, DerefMut, BitOr, BitAnd};
use std::fmt::{self, Debug};

use super::span::Span;
use super::style::Style;

#[derive(Clone, PartialEq)] // TODO: Eq, PartialOrd, Ord, Hash
pub struct SpannedAndStyled<T> {
    pub inner: T,
    pub span: Span,
    pub style: Style,
}

impl<T: Debug> Debug for SpannedAndStyled<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if fmt.alternate() {
            self.inner.fmt(fmt)
        } else {
            fmt.debug_struct(std::any::type_name::<Self>())
                .field("inner", &self.inner)
                .field("span", &self.span)
                .field("style", &self.style)
                .finish()
        }
    }
}

impl<T> SpannedAndStyled<T> {
    pub fn map<B>(self, f: impl FnOnce(T) -> B) -> S<B> {
        let SpannedAndStyled { inner, span, style } = self;
        SpannedAndStyled { inner: f(inner), span, style }
    }
}

pub type S<T> = SpannedAndStyled<T>;

impl<T> Deref for S<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for S<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// Spanned | Span
impl<T> BitOr<Span> for S<T> {
    type Output = Span;

    fn bitor(self, s: Span) -> Span {
        self.span | s
    }
}

// Spanned & Style
impl<T> BitAnd<Style> for S<T> {
    type Output = Style;

    fn bitand(self, s: Style) -> Style {
        self.style & s
    }
}

// Span | Spanned
impl<T> BitOr<S<T>> for Span {
    type Output = Span;

    fn bitor(self, s: S<T>) -> Span {
        self | s.span
    }
}


// Style & Spanned
impl<T> BitAnd<S<T>> for Style {
    type Output = Style;

    fn bitand(self, s: S<T>) -> Style {
        self & s.style
    }
}

// Spanned | Spanned
impl<A, B> BitOr<S<A>> for S<B> {
    type Output = Span;

    fn bitor(self, s: S<A>) -> Span {
        self.span | s.span
    }
}

// Spanned & Spanned
impl<A, B> BitAnd<S<A>> for S<B> {
    type Output = Style;

    fn bitand(self, s: S<A>) -> Style {
        self.style & s.style
    }
}
