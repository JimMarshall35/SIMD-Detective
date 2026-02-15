use clap::Parser;
use clap::builder::Str;
use std::hash::Hash;
use std::os::linux::raw;
use std::{fs::File, str::FromStr};
use std::io::{Lines, prelude::*};
use std::path::PathBuf;
use regex::Regex;
use std::collections::HashSet;
use glob::glob;

//static mut cpuids: Vec<String> = Vec::new();

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
impl ToString for Signature {
    fn to_string(&self) -> String {
        let mut r = String::new();
        let ret = self.rettype.clone() + " ";
        let name = self.name.clone() + "(";
        r += "\x1b[1m\x1b[35m"; /* bold on, magenta */
        r += &ret;
        r += "\x1b[33m";
        r += &name;
        r += "\x1b[0m\x1b[1m";

        for i in 0..self.params.len() {
            let param = &self.params[i];
            r += "\x1b[35m";
            r += &param.arg_type;
            r += "\x1b[0m\x1b[1m";
            r += " ";
            r += &param.name;
            if(i != self.params.len() - 1)
            {
                r += ", ";
            }
        }
        r += "\x1b[33m\x1b[1m";
        r += ")";
        r += "\x1b[0m";
        r
    }
}


#[derive(Eq, Hash, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Intrinsic {
    instruction: String,
    signature: Signature, 
    synopsis: Synopsis,
    description: String,
    operation: String,
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

    /// if set, these files will be excluded
    #[arg(short, long)]
    exclude_list: Vec<std::path::PathBuf>,

    /// files to include, mutually exclusive with recursive_file_extensions option
    #[arg(short, long)]
    files: Vec<std::path::PathBuf>,

    /// List the cpuid flags necessary for the intrinsics used in the files specified
    #[arg(long, default_value = "true")]
    list_cpuid_flags: bool,

    /// list the intrinsics used in the files specified
    #[arg(long, default_value = "false")]
    list_intrinsics: bool,

    /// list the intrinsics used and a description of them
    #[arg(long, default_value = "false")]
    list_info: bool,

    /// list the intrinsics used and pseudocode of how they work
    #[arg(long, default_value = "false")]
    list_operation: bool,

    /// list the intrinsics used and cpuid flags each one requires (different from list_cpuid_flags which lists the required flags for all code files scanned)
    #[arg(long, default_value = "false")]
    list_cpuid: bool,

    #[arg(long, short, default_value = "/usr/share/simd-detective-intrinsics.json")]
    data_file: PathBuf,
}

fn load_file(file_path: &PathBuf) -> String
{
    let mut s = String::new();
    let display = file_path.display();
    let mut file = match File::open(&file_path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => s,
    }
}

fn load_intrinsics_data(path: &PathBuf) -> Intrinsics {
    let data_json = load_file(path);
    let data: Result<Intrinsics, serde_json::Error> = serde_json::from_str(&data_json);
    return match data {
        Err(why) => panic!("couldn't read json data file! {}: {}", path.display(), why),
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
    let raw_data: Intrinsics  = load_intrinsics_data(&args.data_file);
    let by_name = build_intrinsic_name_hashmap(&raw_data);
    let num = by_name.keys().len();

    if !args.files.is_empty() {
        let mut cpuids: HashSet<String> = HashSet::new();
        let mut intrinsics: HashSet<&Intrinsic> = HashSet::new();
        
        for file_path in args.files {
            let fp_string = file_path.to_str().unwrap();
            if fp_string.contains("*") {
                for entry in glob(fp_string).expect("Failed to read glob pattern") {
                    let glob_path = match entry {
                        Ok(path) => path,
                        Err(e) => !panic!("{:?}", e),
                    };
                    let file_contents = remove_comments(&load_file(&file_path));
                    let res = check_for_intrinsics(&file_contents, &by_name);
                    intrinsics.extend(res);
                }
                
            }
            else {
                let file_contents = remove_comments(&load_file(&file_path));
                let res = check_for_intrinsics(&file_contents, &by_name);
                intrinsics.extend(res);
            }
        }
        for intrinsic in intrinsics {
            for s in &intrinsic.synopsis.cpuids {
                cpuids.insert(s.to_string());
            }
            if args.list_intrinsics || args.list_info || args.list_operation {
                println!("{}\n", &intrinsic.signature.to_string());
            }
            if args.list_info {
                println!("{}\n", &intrinsic.description);
            }
            if args.list_operation {
                println!("{}\n", &intrinsic.operation);
            }
            if args.list_cpuid_flags {
                print!("\x1b[31m");
                for id in &intrinsic.synopsis.cpuids {
                    print!("{} ", id);
                }
                print!("\x1b[0m\n");
            }
            println!("");
            //if args.lis
        }
        if args.list_cpuid_flags {
            for cpuid in cpuids {
                println!("{}", cpuid);
            }
        }
    }

}
