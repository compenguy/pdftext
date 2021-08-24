use std::env::args;

use anyhow::{Context, Result};

use pdf::content::{Op, TextDrawAdjusted};
use pdf::object::Resolve;
use pdf::{backend::Backend, file::File};

use crate::text::FontCache;

mod text;

pub fn extract_pdf<B: Backend>(pdf: &File<B>) -> Result<String> {
    let mut doc_text = String::new();
    println!("\nDocument has {} pages.", pdf.num_pages());
    for page in pdf.pages() {
        let page = page.with_context(|| "Failed to extract page from pdf")?;
        let mut fc: FontCache = FontCache::new();
        for (name, font) in page.resources.iter().flat_map(|r| &r.fonts) {
            fc.add_font(name, pdf.get(*font).with_context(|| "Failed to extract font data from pdf")?);
        }
        for gs in page
            .resources
            .iter()
            .flat_map(|r| r.graphics_states.values())
        {
            if let Some((font, _)) = gs.font {
                let font = pdf.get(font).with_context(|| "Failed to extract font data from pdf's current graphics state")?;
                fc.add_font(font.name.clone(), font);
            }
        }

        for op in page.contents.iter().flat_map(|c| &c.operations) {
            match op {
                Op::GraphicsState { name } => {
                    if let Some(ref res) = page.resources {
                        if let Some((font, _)) =
                            res.graphics_states.get(name).and_then(|gs| gs.font)
                        {
                            let font = pdf.get(font).with_context(|| "Failed to look up cached font information for graphics state's current font")?;
                            fc.select_font(Some(font.name.clone()));
                        }
                    }
                }
                Op::TextFont { name, .. } => fc.select_font(Some(name.to_string())),
                Op::TextDraw { text } => {
                    if let Some(text) = fc.decode(&text.data) {
                        doc_text.push_str(&text);
                    }
                }
                Op::TextDrawAdjusted { array } => {
                    for data in array {
                        if let TextDrawAdjusted::Text(text) = data {
                            if let Some(text) = fc.decode(&text.data) {
                                doc_text.push_str(&text);
                            }
                        }
                    }
                }
                Op::TextNewline => {
                    doc_text.push('\n');
                }
                _ => {}
            }
        }
    }
    Ok(doc_text)
}

pub fn extract_pdf_bytes(pdf_bytes: &[u8]) -> Result<String> {
    let file = File::from_data(pdf_bytes.to_vec()).with_context(|| "Failed while parsing pdf content")?;
    extract_pdf(&file)
}

fn main() -> Result<()> {
    let path = args().nth(1).expect("Usage: pdftext <filename>");
    println!("Extracting text from file at {}", path);
    let file = File::<Vec<u8>>::open(&path).with_context(|| "Failed while parsing pdf content")?;
    println!("{}", extract_pdf(&file).with_context(|| "Failed while extracting pdf text content")?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_pdf_extraction() {
        // Credit: https://brendanzagaeski.appspot.com/0004.html
        let text = b"%PDF-1.1
%\xC2\xA5\xC2\xB1\xC3\xAB

1 0 obj
  << /Type /Catalog
     /Pages 2 0 R
  >>
endobj

2 0 obj
  << /Type /Pages
     /Kids [3 0 R]
     /Count 1
     /MediaBox [0 0 300 144]
  >>
endobj

3 0 obj
  <<  /Type /Page
      /Parent 2 0 R
      /Resources
       << /Font
           << /F1
               << /Type /Font
                  /Subtype /Type1
                  /BaseFont /Times-Roman
               >>
           >>
       >>
      /Contents 4 0 R
  >>
endobj

4 0 obj
  << /Length 54 >>
stream
  BT
    /F1 18 Tf
    0 0 Td
    (Hello World) Tj
  ET
endstream
endobj

xref
0 5
0000000000 65535 f 
0000000018 00000 n 
0000000077 00000 n 
0000000178 00000 n 
0000000457 00000 n 
trailer
  <<  /Root 1 0 R
      /Size 5
  >>
startxref
565
%%EOF";
        let out = extract_pdf_bytes(text).expect("Failed to extract pdf text from supplied byte data");
        assert_eq!(out, "Hello World");
    }
}
