use std::collections::HashMap;
use std::convert::TryInto;

use pdf::encoding::BaseEncoding;
use pdf::font::{Font, ToUnicodeMap};
use pdf::object::RcRef;

pub(crate) struct FontInfo {
    font: RcRef<Font>,
    cmap: ToUnicodeMap,
}

impl FontInfo {
    pub(crate) fn decode(&self, data: &[u8]) -> Option<String> {
        let mut out = String::new();
        if let Some(base_encoding) = self.font.encoding().map(|enc| enc.base.clone()) {
            match base_encoding {
                BaseEncoding::IdentityH => {
                    for s in data
                        .windows(2)
                        .filter_map(|w| {
                            let a_w: Option<[u8; 2]> = w.try_into().ok();
                            a_w
                        })
                        .map(u16::from_be_bytes)
                        .filter_map(|cp| self.cmap.get(cp))
                    {
                        out.push_str(s);
                    }
                }
                _ => {
                    for b in data {
                        match self.cmap.get(*b as u16) {
                            Some(s) => out.push_str(s),
                            None => out.push(*b as char),
                        }
                    }
                }
            }
            Some(out)
        } else {
            None
        }
    }
}

pub(crate) struct FontCache {
    fonts: HashMap<String, FontInfo>,
    selected: Option<String>,
}

impl FontCache {
    pub(crate) fn new() -> Self {
        FontCache {
            fonts: HashMap::new(),
            selected: None,
        }
    }

    pub(crate) fn add_font(&mut self, name: impl Into<String>, font: RcRef<Font>) {
        if let Some(to_unicode) = font.to_unicode() {
            self.fonts.insert(
                name.into(),
                FontInfo {
                    font,
                    cmap: to_unicode.expect("Failed to convert font to unicode"),
                },
            );
        }
    }

    pub(crate) fn select_font(&mut self, name: Option<String>) {
        self.selected = name;
    }

    pub(crate) fn decode(&self, data: &[u8]) -> Option<String> {
        self.selected
            .as_ref()
            .and_then(|s| self.fonts.get(s.as_str()))
            .and_then(|fi| fi.decode(data))
    }
}
