use clap::Parser;
use clap::builder::Str;
use std::hash::Hash;
use std::os::linux::raw;
use std::{fs::File, str::FromStr};
use std::io::{Lines, prelude::*};
use std::path::PathBuf;
use regex::Regex;
use std::collections::HashSet;

static mut cpuids: Vec<String> = Vec::new();

use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct FunctionParam {
    name: String,

    #[serde(rename = "type")]
    arg_type: String
}

#[derive(Eq, Hash, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct  Synopsis {
    cpuids: Vec<String> 
}

#[derive(Eq, Hash, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct  Signature {
    name: String,
    params: Vec<FunctionParam>,
    rettype: String
}

#[derive(Eq, Hash, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Intrinsic {
    instruction: String,
    signature: Signature, 
    synopsis: Synopsis
}

/*
AVX_512
Other
SSE_ALL
AVX_ALL
SVML
MMX
AMX
*/
#[derive(Eq, Hash, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Intrinsics {
    #[serde(rename = "AVX_512")]
    avx_512: Vec<Intrinsic>,

    #[serde(rename = "Other")]
    other: Vec<Intrinsic>,

    #[serde(rename = "SSE_ALL")]
    sse_all: Vec<Intrinsic>,

    #[serde(rename = "AVX_ALL")]
    avx_all: Vec<Intrinsic>,

    #[serde(rename = "SVML")]
    svml: Vec<Intrinsic>,

    #[serde(rename = "MMX")]
    mmx: Vec<Intrinsic>,

    #[serde(rename = "AMX")]
    amx: Vec<Intrinsic>,
}


/// Analyse the use of intrinsics in your source code
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    
    /// If set, these files will be found recursively and analysed
    #[arg(short, long)]
    recursive_file_extensions: Vec<String>,

    /// if set, these files will be excluded
    #[arg(long)]
    exclude_list: Vec<std::path::PathBuf>,

    /// files to include, mutually exclusive with recursive_file_extensions option
    #[arg(short, long)]
    files: Vec<std::path::PathBuf>,

    /// startdir
    #[arg(short, long, default_value = ".")]
    startdir: std::path::PathBuf,
}

fn load_file(file_path: &PathBuf) -> String
{
    let mut s = String::new();
    let display = file_path.display();
    println!("{}", file_path.display());
    let mut file = match File::open(&file_path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => s,
    }
}

fn load_intrinsics_data() -> Intrinsics {
    let b =  match PathBuf::from_str("./data/intrinsics.json") {
        Ok(value) => value,
        Err(e) => panic!("Error: {}", e),
    };
    let data_json = load_file(&b);
    let data: Result<Intrinsics, serde_json::Error> = serde_json::from_str(&data_json);
    return match data {
        Err(why) => panic!("couldn't read jso data file! {}: {}", b.display(), why),
        Ok(s) => s
    };
}

fn build_intrinsic_name_hashmap(raw_data: &Intrinsics) -> HashMap<String, Intrinsic> {
    let mut map: HashMap<String, Intrinsic> = HashMap::new();
    for intrinsic in &raw_data.avx_512 {
        map.insert(intrinsic.signature.name.clone(), intrinsic.clone());
    }
    for intrinsic in &raw_data.other {
        map.insert(intrinsic.signature.name.clone(), intrinsic.clone());
    }
    for intrinsic in &raw_data.sse_all {
        map.insert(intrinsic.signature.name.clone(), intrinsic.clone());
    }
    for intrinsic in &raw_data.avx_all {
        map.insert(intrinsic.signature.name.clone(), intrinsic.clone());
    }
    for intrinsic in &raw_data.svml {
        map.insert(intrinsic.signature.name.clone(), intrinsic.clone());
    }
    for intrinsic in &raw_data.mmx {
        map.insert(intrinsic.signature.name.clone(), intrinsic.clone());
    }
    for intrinsic in &raw_data.amx {
        map.insert(intrinsic.signature.name.clone(), intrinsic.clone());
    }
    map
}

fn remove_comments(file_contents: &str) -> String
{
    /* remove single line comments */
    let lines: Vec<&str> = file_contents.split('\n').collect();
    let mut new_string = String::new();
    for l in lines {
        let first_segment= l.split("//").next().unwrap();
        new_string += first_segment;
        new_string += "\n";
    }

    /* remove multi line comments */
    let re = Regex::new(r"\/\*([\s\S]*?)\*\/").unwrap();
    
    let comments: Vec<&str> = re.find_iter(&new_string).map(|m| m.as_str()).collect();
    let mut newer_string = new_string.clone();
    for c in comments {
        newer_string = newer_string.replace(c, "");
        // why can't i use new_string here?! i don't get it
    }
    newer_string
}

fn check_for_intrinsics<'a>(file_contents: &str, by_name: &'a HashMap<String, Intrinsic>) -> HashSet<&'a Intrinsic> {
    let mut r : HashSet<&'a Intrinsic> = HashSet::new();
    for k in by_name {
        let re = Regex::new(k.0).unwrap();
        let itr = re.find_iter(&file_contents).map(|m| m.as_str());
        let occurences = itr.count();
        if occurences > 0 {
            r.insert(k.1);
        }
        
    }

    r
}

fn main()
{
    let args: Args = Args::parse();
    let raw_data: Intrinsics  = load_intrinsics_data();
    let by_name = build_intrinsic_name_hashmap(&raw_data);
    let num = by_name.keys().len();
    print!("num keys {}", num);
    
    if !args.recursive_file_extensions.is_empty() {
        if !args.files.is_empty() {
            println!("Mutually exclusive options -r and -f passed! exiting");
            std::process::exit(1);
        }
        // recursive
    }
    if !args.files.is_empty() {
        // specific files
        for file_path in args.files {
            let file_contents = remove_comments(&load_file(&file_path));
            let res = check_for_intrinsics(&file_contents, &by_name);
            for intrinsic in res {
                println!("{}", &intrinsic.signature.name);
            }
        }
    }

}
