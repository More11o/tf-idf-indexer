
pub struct Lexer<'a> {
    // The lexer doesn't need to own the content but the reference
    //  to content needs to last as long as the Lexer is around.
    content: &'a [char]
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self{
        Lexer { content }
    }
    
    ///
    /// I'm a re-slicer.
    /// 
    /// I reslice the str to remove tokens from the left when i match
    /// the correct pattern.
    /// 
    /// Eg.
    ///     1999
    ///     text
    ///     text1999
    ///     &
    /// 
    fn next_token(&mut self) -> Option<&'a [char]> {
 
        // Trim left hand whitespace
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }

        // EOF?
        if self.content.len() == 0 {
            return None
        }

        // is the first char a number?
        if self.content[0].is_numeric() {
            return Some(self.slice_while(|x| x.is_numeric()))
        }

        // is the first char alphabetic?
        if self.content[0].is_alphabetic() {
            return Some(self.slice_while(|x| x.is_alphanumeric()))
        }

        // i get here if im a random char (eg. emoji).
        return Some(self.slice(1))
    }
    
    // Return [char; n] and remove it from self.contents
    fn slice(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];

        token
    }

    // todo: look up predicates! Rust book covers FnMut but needs more reading.
    fn slice_while<P>(
        &mut self, mut predicate: P
    ) -> &'a [char] where P: FnMut(&char) -> bool {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.slice(n)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];
 
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}