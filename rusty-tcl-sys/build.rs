extern crate bindgen;
#[macro_use]
extern crate failure;

use std::{collections::HashMap, env, fs, io::Read, path::PathBuf, process::Command};

#[cfg(target_family = "unix")]
fn get_tcl_config_paths() -> Result<String, failure::Error> {
    Ok(String::from_utf8(
        Command::new("locate").arg("tclConfig.sh").output()?.stdout,
    )?)
}

#[cfg(not(target_family = "unix"))]
fn get_tcl_config_paths() -> Result<String, Error> {
    compile_error!("Currently, rusty-tcl-sys only supports *nix. Your help in supporting more platforms would be greatly appreciated!")
}

fn read_tcl_config() -> Result<Vec<String>, failure::Error> {
    get_tcl_config_paths()?
        .lines()
        .filter_map(|path| {
            let mut contents = String::new();
            fs::File::open(path)
                .unwrap()
                .read_to_string(&mut contents)
                .unwrap();
            let mut lines = contents
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.starts_with("#"))
                .peekable();

            if lines
                .peek()
                .map(|line| line.starts_with("."))
                .unwrap_or(true)
            {
                None
            } else {
                Some(lines.map(|line| line.to_owned()).collect())
            }
        })
        .next()
        .ok_or_else(|| format_err!("Could not find a valid tclConfig.sh"))
}

fn parse_tcl_config() -> Result<HashMap<String, String>, failure::Error> {
    Ok(read_tcl_config()?
        .into_iter()
        .filter_map(|line| {
            line.find('=')
                .map(|equals_index| line.split_at(equals_index))
                .map(|(left, right)| {
                    let left = left.to_owned();
                    let mut right = right.to_owned();

                    if !right.is_empty() {
                        right.remove(0); // remove the =
                    }

                    if !right.is_empty() {
                        right.remove(0); // remove the left '
                        right.pop(); // remove the right '
                    }

                    (left, right)
                })
        })
        .collect())
}

fn main() {
    let vars = parse_tcl_config().unwrap();

    vars.get("TCL_LIB_SPEC")
        .expect("Could not read $TCL_LIB_SPEC")
        .split_whitespace()
        .for_each(|arg| println!("cargo:rustc-flags={} {}", &arg[..2], &arg[2..]));

    let header_dir = PathBuf::from(
        &vars.get("TCL_INCLUDE_SPEC")
            .expect("Could not read $TCL_INCLUDE_SPEC")[2..],
    );
    let bindings = bindgen::Builder::default()
        .header(header_dir.join("tcl.h").to_str().unwrap())
        .trust_clang_mangling(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("Could not read $OUT_DIR"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
