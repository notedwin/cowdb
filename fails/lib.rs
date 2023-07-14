//!
#![warn(missing_docs, rust_2018_idioms, missing_debug_implementations)]

// 'a how long does something live for, guess the lifetime, similar to T
pub struct StrSplit<'a>{
    remainder: &'a str,
    delimiter: &'a str,
}


impl StrSplit<'_> {
    pub fn new(haystack: &str, delimiter: &str) -> Self {
        Self {
            remainder: haystack,
            delimiter,
        }
    }
}

impl<'a> Iterator for StrSplit<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_delim) = self.remainder.find(self.delimiter){
            let until_delimeter = &self.remainder[..next_delim];
            self.remainder = &self.remainder[(next_delim + self.delimiter.len())..];
            Some(until_delimeter)
        } else if self.remainder.is_empty() {
            // TODO: BUG
            None
        } else {
            let rest = self.remainder;
            self.remainder = &[];
            Some(rest)
        }
    }
}

#[test]
fn works(){
    let haystack = "a b c d e";
    let letters = StrSplit::new(haystack, " ");
}