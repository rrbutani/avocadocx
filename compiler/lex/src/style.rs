use docx_rs::{Justification, ParagraphStyle, RunProperty};
use std::ops::BitAnd;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    pub prop: RunProperty,
    pub paragraph_style: Option<ParagraphStyle>,
    pub alignment: Option<Justification>,
}

impl Style {
    pub fn intersect(&self, other: &Style) -> Style {
        fn same_or_none<T: PartialEq + Clone>(a: &Option<T>, b: &Option<T>) -> Option<T> {
            if a == b {
                a.clone()
            } else {
                None
            }
        }

        macro_rules! prop {
            ($field:ident) => {
                same_or_none(&self.prop.$field, &other.prop.$field)
            };
        }

        Style {
            prop: RunProperty {
                sz: prop!(sz),
                sz_cs: prop!(sz_cs),
                color: prop!(color),
                highlight: prop!(highlight),
                vert_align: prop!(vert_align),
                underline: prop!(underline),
                bold: prop!(bold),
                bold_cs: prop!(bold_cs),
                italic: prop!(italic),
                italic_cs: prop!(italic_cs),
                vanish: prop!(vanish),
                spacing: prop!(spacing),
                fonts: prop!(fonts),
                text_border: prop!(text_border),
                del: prop!(del),
                ins: prop!(ins),
            },
            paragraph_style: match (&self.paragraph_style, &other.paragraph_style) {
                (sty @ Some(a), Some(b)) => {
                    if a == b {
                        sty.clone()
                    } else {
                        Some(ParagraphStyle {
                            val: "Normal".to_string(),
                        })
                    }
                }
                _ => None,
            },
            alignment: match (&self.alignment, &other.alignment) {
                (jus @ Some(a), Some(b)) if a == b => jus.clone(),
                _ => None,
            },
        }
    }
}

impl BitAnd for Style {
    type Output = Style;

    fn bitand(self, rhs: Self) -> Style {
        self.intersect(&rhs)
    }
}
