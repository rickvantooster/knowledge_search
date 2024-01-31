use rust_stemmers::{Stemmer, Algorithm};

pub struct Lexer<'a> {
    buffer: &'a[char],
    stemmer: Option<Stemmer>
}

impl<'a> Lexer<'a>{
    pub fn new(buffer: &'a [char]) -> Self {
        Lexer {
            buffer,
            stemmer: None

        }
    }


    pub fn new_stemmed(buffer: &'a [char]) -> Self {
        Lexer {
            buffer,
            stemmer: Some(Stemmer::create(Algorithm::English))

        }
    }

    pub fn next_token(&mut self) -> Option<String> {
        self.skip_whitespaces();
        if self.buffer.is_empty(){
            return None;

        }
        if self.buffer[0].is_numeric() {
            return Some(self.chop_while(|x| x.is_numeric()).iter().collect());
        }
        if self.buffer[0].is_alphabetic() {
            let token = self.chop_while(|x| x.is_alphanumeric());
            let temp: String = token.iter().collect();
            let temp = temp.to_uppercase();
            
            //let temp: String = .iter().collect::<String>().to_uppercase();

            let result = match &self.stemmer {
                Some(stemmer) => {
                    stemmer.stem(&temp).to_string()
                },
                None => temp

            };

            return Some(result);

        }

        Some(self.chop(1).iter().collect::<String>().to_uppercase())
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token =  &self.buffer[0..n];
        self.buffer = &self.buffer[n..];
        token

    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char] where P: FnMut(&char) -> bool {
        let mut n = 0;
        while n < self.buffer.len() && predicate(&self.buffer[n]) {
            n += 1;
        }
        self.chop(n)
    }



    fn skip_whitespaces(&mut self) {
        while !self.buffer.is_empty() && self.buffer[0].is_whitespace() {
            self.buffer = &self.buffer[1..];

        }

    }

}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
