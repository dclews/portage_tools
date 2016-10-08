#![feature(plugin, try_from)]
#![plugin(clippy)]

extern crate regex;
use regex::Regex;

use std::path::PathBuf;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::collections::HashSet;
use std::fmt;
use std::convert;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct AtomEnvironmentMap {
    path: PathBuf,
    name: String,
    atoms: HashSet<Atom>,
}
impl AtomEnvironmentMap {
    pub fn new(env_name: &str) -> AtomEnvironmentMap {
        let mut path = PathBuf::from("/etc/portage/package.env");
        path.push(env_name);
        let name = path.to_str().unwrap().to_owned();
        AtomEnvironmentMap {
            path: path,
            atoms: HashSet::new(),
            name: name,
        }
    }
    pub fn reload(&mut self) -> Result<(), String> {
        let f = try!(self.load_file());
        let reader = BufReader::new(f);

        for line in reader.lines() {
            let l = line.expect("Failed to read line in environment file");
            let atom_map: Vec<&str> = l.split_whitespace().collect();
            if let Some(atom) = atom_map.get(0) {
                let atom = Atom::try_from(atom).unwrap();
                self.atoms.insert(atom);
//                if !atom.in_world_set() {
//                    panic!("Atom {} is not in world set", atom);
//                }
//                println!("{}", atom);
            }
        }
        Ok(())
    }
    fn load_file(&self) -> Result<File, String> {
        if self.path.exists() {
            match File::open(&self.path) {
                Ok(f) => Ok(f),
                Err(e) => Err(format!("Failed to open environment file '{:?}': {}", self.path, e)),
            }
        }
        else {
            match File::create(&self.path) {
                Ok(f) => Ok(f),
                Err(e) => Err(format!("Failed to create environment file '{:?}': {}", self.path, e)),
            }
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn atoms(&self) -> &HashSet<Atom> {
        &self.atoms
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum AtomVersionOperator {
    Lt,
    Lte,
    Eq,
    Gte,
    Gt,
}
impl<'a> convert::TryFrom<&'a str> for AtomVersionOperator {
    type Err = String;
    fn try_from(s: &'a str) -> Result<Self, String> {
        match s {
            "<" => Ok(AtomVersionOperator::Lt),
            "<=" => Ok(AtomVersionOperator::Lte),
            "=" => Ok(AtomVersionOperator::Eq),
            ">=" => Ok(AtomVersionOperator::Gte),
            ">" => Ok(AtomVersionOperator::Gt),
            _ => Err(format!("Invalid atom version operator '{}'", s)),
        }
    }
}
impl fmt::Display for AtomVersionOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AtomVersionOperator::Lt => write!(f, "<"),
            AtomVersionOperator::Lte => write!(f, "<="),
            AtomVersionOperator::Eq => write!(f, "="),
            AtomVersionOperator::Gte => write!(f, ">="),
            AtomVersionOperator::Gt => write!(f, ">"),
        }
    }
}
pub type AtomVersionConstraint = (AtomVersionOperator, String);

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Atom {
    category: String,
    package: String,
    version_constraint: Option<AtomVersionConstraint>,
//    repository_constraint: Option<String>,
}
impl Atom {
    pub fn new(category: String,
               package: String,
               version_contraint: Option<AtomVersionConstraint>) -> Atom {
        Atom {
            category: category,
            package: package,
            version_constraint: version_contraint
        }
    }
    pub fn in_world_set(&self) -> bool {
        let f = File::open("/var/lib/portage/world").expect("Failed to open world file");
        let buf_reader = BufReader::new(f);
        let category_package = format!("{}/{}", self.category, self.package);
        println!("Debug '{}'", category_package);
        buf_reader.lines().any(|l| l.unwrap() == category_package)
    }
}
impl<'a> convert::TryFrom<&'a str> for Atom {
    type Err = String;
    fn try_from(s: &'a str) -> Result<Self, String> {
        let versioned_atom_regex = Regex::new(r"((?:=|<|>)+)(.+)/(.+)-(\d.+)")
            .expect("Failed to build versioned atom parsing regex.");
        let non_versioned_atom_regex = Regex::new(r"(.+)/(.+)")
            .expect("Failed to build atom parsing regex.");

        if let Some(caps) = versioned_atom_regex.captures(s) {
            let version_operator = try!(AtomVersionOperator::try_from(caps.at(1).unwrap()));
            let category = caps.at(2).unwrap();
            let package = caps.at(3).unwrap();
            let version = caps.at(4).unwrap();
            Ok(Atom::new(category.to_owned(), package.to_owned(),
                         Some((version_operator, version.to_owned())))
            )
        }
        else if let Some(caps) = non_versioned_atom_regex.captures(s) {
            let category = caps.at(1).unwrap();
            let package = caps.at(2).unwrap();
            Ok(Atom::new(category.to_owned(), package.to_owned(), None))
        }
        else {
            Err(format!("Invalid package atom '{}'", s))
        }
    }
}
impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((ref version_constraint_op, _)) = self.version_constraint {
            write!(f, "{}", version_constraint_op).expect("Failed to write to stdout");
        }
        write!(f, "{}/{}", self.category, self.package).expect("Failed to write to stdout");
        if let Some((_, ref version)) = self.version_constraint {
            write!(f, "-{}", version).expect("Failed to write to stdout");
        }
        Ok(())
    }
}