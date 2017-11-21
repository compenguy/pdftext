#[macro_use]
extern crate error_chain;

extern crate pdf;

use std::env::args;
use pdf::file::File;
use pdf::backend::Backend;
use pdf::primitive::Primitive;

error_chain! {
    foreign_links {
        Pdf(::pdf::Error);
        Utf8(::std::string::FromUtf8Error);
    }
    errors {
        AnError(t: String) {
            description("An error occurred.")
            display("An error occurred: {}", t)
        }
    }
}

pub fn pdf_primitive_to_string(primitive: &Primitive) -> Result<String> {
    let pdftext = match *primitive {
        Primitive::String(ref pdfstring) => pdfstring.clone().into_string()?,
        _                                 => String::new(),
    };
    Ok(pdftext)
}

pub fn extract_pdf<B: Backend>(pdf: &File<B>) -> Result<String> {
    let mut doc_text = String::new();
    println!("\nDocument has {} pages.", pdf.get_num_pages()?);
    for page in pdf.pages() {
        for content in page.contents.iter().as_ref() {
            for operation in content.operations.iter().as_ref() {
                println!("Adding doc text...");
                match operation.operator.as_ref() {
                    "Tj" | "TJ" | "\"" | "'" => {
                        for primitive in &operation.operands {
                            let pdftext = pdf_primitive_to_string(primitive)?;
                            doc_text += &pdftext;
                        }
                    }
                    other                    => println!("Unhandled operator: {}", other),
                }
            }
        }
    }
    Ok(doc_text)
}

pub fn extract_pdf_bytes(pdf_bytes: &[u8]) -> Result<String> {
    let file = File::new(pdf_bytes.to_vec());
    extract_pdf(&file)
}

quick_main!(|| -> Result<()> {
    let path = args().nth(1).expect("Usage: pdftext <filename>");
    println!("Extracting text from file at {}", path);
    let file = File::<Vec<u8>>::open(&path)?;
    println!("{}", extract_pdf(&file)?);
    Ok(())
});

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
  << /Length 55 >>
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
        assert_eq!(extract_pdf_bytes(text).unwrap().trim(), "Hello World");
    }
}
