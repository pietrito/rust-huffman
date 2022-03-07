
mod huffman;
use std::error::Error;
use std::path::Path;
use clap::{Arg, App};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Rust Huffman Compression")
                          .version("1.0")
                          .author("User420")
                          .arg(Arg::with_name("mode")
                            .short("m")
                            .long("mode")
                            .takes_value(true)
                            .help("Action to perform")
                            .required(true))
                          .arg(Arg::with_name("in_path")
                            .short("i")
                            .long("inpath")
                            .value_name("FILE")
                            .help("The path of the input file to process")
                            .required(true)
                            .takes_value(true))
                          .arg(Arg::with_name("out_path")
                            .help("The path of the output")
                            .index(1))
                          .arg(Arg::with_name("verbose")
                            .short("v")
                            .long("verbose")
                            .takes_value(false)
                            .help("Sets verbosity on"))
                          .arg(Arg::with_name("N")
                            .help("Number of iterations"))
                          .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let iterations = matches.value_of("N").unwrap_or("1").parse::<u64>().unwrap();

    let path_in = String::from(matches.value_of("in_path").unwrap());
    // Build the out path
    let mut path_out = match matches.is_present("out_path") {
        true => String::from(matches.value_of("out_path").unwrap()),
        false => {
            let raw = Path::new(&path_in);
            let folder = raw.parent().unwrap();
            let file_stem = raw.file_stem().unwrap();
            let path = folder.join(file_stem.to_str().unwrap());

            let str_path = String::from(path.to_str().unwrap());
            str_path
        }
    };

    let mut compress = false;
    match matches.value_of("mode").unwrap() {
        "c" | "compress" => {
            compress = true;
            path_out.push_str(".huff");
        },
        "d" | "decompress" => {
            path_out.push_str(".dhuff");
        }
        _ => panic!("Invalid mode"),
    };

    for i in 0..iterations {
        path_out.push_str(&i.to_string());
        if compress {
            huffman::compress(&path_in, &path_out)?;
        } else {
            huffman::decompress(&path_in, &path_out)?;
        }
    }

    Ok(())

}
