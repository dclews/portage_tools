#![feature(plugin, try_from)]
#![plugin(clippy)]

extern crate portage_tools;
use portage_tools::{Atom, AtomEnvironmentMap};
use std::path::Path;
use std::fs;
use std::convert::TryFrom;

pub fn main() {
    let mut env_maps: Vec<AtomEnvironmentMap> = vec![];

    for entry in fs::read_dir(Path::new("/etc/portage/package.env"))
        .expect("Failed to read package.env/") {
        let env_file = entry.expect("Failed to read environment mapping from package.use");
        let env_path = env_file.path();
        if !env_path.is_dir() {
            let env_name =  env_path.file_name().unwrap();
            env_maps.push(AtomEnvironmentMap::new(env_name.to_str().unwrap()));
        }
    }
    for mut env in &mut env_maps {
        env.reload().expect("Failed to load environment mapping");
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(arg) = args.get(0) {
        match arg.as_str() {
            "list" => {
                for env_map in &env_maps {
                    println!("{}: {} packages", env_map.name(), env_map.atoms().len())
                }
            },
            "set" => {
                let atom = args.get(1).unwrap();
                let atom = Atom::try_from(atom).expect("Invalid atom");
                for conflict in env_maps.iter().filter(|em| em.atoms().iter().any(|a| *a == atom)) {
                    panic!("[Error] Atom '{}' already exists in environment mapping '{}'",
                             atom,
                             conflict.name(),
                    );
                }
           }
            _ => panic!("Unknown command"),
        }
    }
}