use std::{
    fs::{self, File, DirEntry},
    io::{ BufReader, BufWriter, Result },
    path::{Path, PathBuf},
    collections::HashMap,
};

use xml::reader::{
    EventReader,
    XmlEvent::Characters
};

pub mod lexer;
use lexer::Lexer;

type TermFreq = HashMap::<String, usize>;
type TermFreqIndex = HashMap<PathBuf, TermFreq>;

///
/// Creates an JSON file using containing the TermFrequencys of all files in a given path.
/// 
/// create-index("test", "~/my_files")
/// 
pub fn create_index(dir_path: &str, filename: &str) -> std::io::Result<()>{
    let mut tf_index = TermFreqIndex::new();

    let path = Path::new(dir_path);

    print!("Indexing {dir_path} ... ");

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        
        let tf = index_document(&entry)?;

        tf_index.insert(entry.path(), tf); 

    };
    
    index_to_json(filename, &tf_index)?;

    Ok(())
}

pub fn serve(index_filename: &str) {
    let _ = load_index_file(index_filename);
    todo!()
}

fn load_index_file(filename: &str) -> TermFreqIndex {
    let contents = json_to_index(filename).unwrap();

    contents

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