use std::{env, process};
use tf_idf_indexer;

fn print_usage() {
    println!("Invalid arguments");
    println!(" Usage:");
    println!("  tf_idf_indexer index directory_path output_file");
    println!("  tf_idf_indexer serve input_file");
}
fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        4 => if args[1] == "index" { tf_idf_indexer::create_index(&args[2], &args[3]).unwrap(); },
        3 => if args[1] == "serve" { 
            tf_idf_indexer::serve(&args[2]).unwrap_or(println!("Unable to serve index file."));
            process::exit(1);
         },
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}