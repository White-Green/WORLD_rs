use bindgen::callbacks::ParseCallbacks;
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use std::env;
use std::path::Path;

const WORLD_BASE_DIR: &str = "World";
const WORLD_FILE_NAMES: &[&str] = &["cheaptrick", "codec", "common", "d4c", "dio", "fft", "harvest", "matlabfunctions", "stonemask", "synthesis", "synthesisrealtime"];

fn main() {
    let world_src_dir = Path::new(WORLD_BASE_DIR).join("src");
    generate_bindgen(&world_src_dir);
    for file_name in WORLD_FILE_NAMES {
        cc::Build::new().cpp(true).file(&world_src_dir.join(file_name).with_extension("cpp")).include(&world_src_dir).compile(file_name);
    }
}

fn generate_bindgen(world_src_dir: impl AsRef<Path>) {
    let world_src_dir = world_src_dir.as_ref();
    let world_header_dir = world_src_dir.join("world");
    WORLD_FILE_NAMES
        .iter()
        .fold(bindgen::builder(), |builder, &entry| builder.header(world_header_dir.join(entry).with_extension("h").to_str().unwrap()))
        .clang_arg(format!("-I{}", world_src_dir.display()))
        .clang_arg("-fparse-all-comments")
        .derive_copy(false)
        .parse_callbacks(Box::new(Cb))
        .generate()
        .unwrap()
        .write_to_file(format!("{}/bindgen.rs", env::var("OUT_DIR").unwrap()))
        .unwrap();
}

#[derive(Debug)]
struct Cb;

impl ParseCallbacks for Cb {
    fn process_comment(&self, comment: &str) -> Option<String> {
        static HYPHEN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^\s*-{5,}\s*$").unwrap());
        let comment = HYPHEN_REGEX.replace_all(comment, "");
        static INOUT_MULTILINE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^(.*\S.*:.*\S.*)$\n *([^\s\w]|[^:\n]+$)").unwrap());
        let comment = INOUT_MULTILINE_REGEX.replace_all(&comment, "$1 $2");
        static INOUT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(In|Out)put:").unwrap());
        let comment = INOUT_REGEX.replace_all(&comment, "\n$0");
        static INOUT_BODY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)((?:In|Out)put:\n(?:-.+\n)*)( *[^-\s]+)").unwrap());
        let mut comment = comment.to_string();
        while let Cow::Owned(s) = INOUT_BODY_REGEX.replace_all(&comment, "$1- $2") {
            comment = s;
        }
        static BRACKET_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\[\]]").unwrap());
        let comment = BRACKET_REGEX.replace_all(&comment, r"\$0");
        static URL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://\w+(\.\w+)*(/[\w.]+)*").unwrap());
        let comment = URL_REGEX.replace_all(&comment, "<$0>");
        Some(comment.into_owned())
    }
}
