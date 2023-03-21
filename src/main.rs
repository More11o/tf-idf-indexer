use std::{
    fs::{self, File, DirEntry},
    io::{ BufReader, BufWriter, Result },
    path::{Path, PathBuf},
    collections::HashMap,
    env
};

use xml::reader::{
    EventReader,
    XmlEvent::Characters
};

use serde_json;

type TermFreq = HashMap::<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

struct Lexer<'a> {
    // The lexer doesn't need to own the content but the reference
    //  to content needs to last as long as the Lexer is around.
    content: &'a [char]
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self{
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

fn index_document (entry: &DirEntry) -> Result<TermFreq> {
    print!("Indexing {:?} ... ", entry.path());
    let content = read_file(entry.path())?
        .chars()
        .collect::<Vec<_>>();

    let mut tf = TermFreq::new();

    for token in Lexer::new(&content) {
        let term = token.into_iter().collect::<String>();
        let freq = tf.entry(term).or_insert(1);
        *freq += 1
    };

    println!("Complete");
    Ok(tf)
}

fn read_file<P: AsRef<Path>>(filepath: P) -> Result<String>{
    // todo: remove panic!() and exit function gracefully.
    // todo: investigate File::metadata to choose how to parse file 
    //  when new file formats are added.
    let file = File::open(filepath)?;
 
    let buf_reader = BufReader::new(file);

    let content = parse_xml(buf_reader)?;

    Ok(content)
}

fn parse_xml(file: BufReader<File>) -> Result<String> {
    // This is probably a noisy way to do this but does allow us to
    // include non Character events in the future.
    // I might be able to do this cleaner with .filter() later on. 
    let parser = EventReader::new(file);
    let mut content = String::new();

    // todo: gracefully handle the unwrap.
    let events = parser.into_iter().map(|event| event.unwrap());
    for event in events {
        match event {
            Characters(text) => {
                content.push_str(&text);
                // This padding is needed by the Lexer to stop
                //  words blending into each other.
                content.push_str(" ");
            },
            _ => continue
        }
    }

    Ok(content)
}


fn index_to_json(filename: &str, index: &TermFreqIndex) -> Result<()> {
    print!("Writing index to: {filename} ... ");
    let file = fs::File::create(filename)?;
    let bw = BufWriter::new(file);

    // serde::to_writer_pretty() is avaiable but the resulting file is
    //  almost double the size.
    serde_json::to_writer(bw, index)?;
    println!("Complete!");

    Ok(())
}

fn json_to_index(filename: &str) -> Result<TermFreqIndex> {
    print!("Reading index from: {filename} ... ");
    let file = File::open(filename)?;
    let br = BufReader::new(file);

    let tfi: TermFreqIndex = serde_json::from_reader(br)?;
    println!("Complete!");

    Ok(tfi)
}

fn main() -> std::io::Result<()>{
    
    let dir_path = "gldocs/gl4";
    let db_filename = "index.json";

    let mut tf_index = TermFreqIndex::new();

    let path = Path::new(dir_path);

    print!("Indexing {dir_path} ... ");

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        
        let tf = index_document(&entry)?;

        tf_index.insert(entry.path(), tf); 

    };
    
    index_to_json(db_filename, &tf_index)?;

    Ok(())
}
   