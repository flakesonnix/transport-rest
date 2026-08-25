//! transport-rest-gen: deterministic binding/model generator.
//!
//! Reads `schema/*.json` (internal representation of the transport.rest API)
//! and emits model sources for the supported target languages. Output is
//! deterministic: same input -> byte-identical output (`--check` verifies).

mod csharp;
mod go;
mod java;
mod meta;
mod schema;
mod ts;

use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Lang {
    TypeScript,
    Go,
    Csharp,
    Java,
    /// Canonical endpoint/provider metadata (JSON), consumed by native clients.
    Meta,
}

#[derive(Parser)]
#[command(
    name = "transport-rest-gen",
    about = "Generate language bindings from the internal schema"
)]
struct Args {
    /// Schema directory containing *.json IR files.
    #[arg(long, default_value = "schema")]
    schema_dir: PathBuf,

    /// Target language.
    #[arg(value_enum)]
    lang: Lang,

    /// Output directory; file name is chosen per language.
    #[arg(long)]
    out: PathBuf,

    /// Verify that the generated output matches the files on disk instead of
    /// writing them (CI drift check).
    #[arg(long)]
    check: bool,
}

impl Lang {
    fn file_name(self) -> &'static str {
        match self {
            Lang::TypeScript => "models.ts",
            Lang::Go => "models_gen.go",
            Lang::Csharp => "Models.Generated.cs",
            Lang::Java => "GeneratedModels.java",
            Lang::Meta => "api-meta.json",
        }
    }
}

fn emit(ir: &schema::Ir, lang: Lang) -> String {
    match lang {
        Lang::TypeScript => ts::emit(ir),
        Lang::Go => go::emit(ir),
        Lang::Csharp => csharp::emit(ir),
        Lang::Java => java::emit(ir),
        Lang::Meta => meta::emit(ir),
    }
}

fn main() {
    let args = Args::parse();
    let ir = match schema::load_ir(&args.schema_dir) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    let code = emit(&ir, args.lang);
    let path = args.out.join(args.lang.file_name());

    if args.check {
        match std::fs::read_to_string(&path) {
            Ok(existing) if existing == code => {
                println!("ok: {}", path.display());
            }
            Ok(_) => {
                eprintln!(
                    "error: {} is stale – run transport-rest-gen to regenerate",
                    path.display()
                );
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("error: cannot read {}: {e}", path.display());
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("error: cannot create {}: {e}", parent.display());
            std::process::exit(2);
        }
    }
    if let Err(e) = std::fs::write(&path, &code) {
        eprintln!("error: cannot write {}: {e}", Path::display(&path));
        std::process::exit(2);
    }
    println!("wrote {}", path.display());
}
