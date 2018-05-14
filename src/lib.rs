#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn nvcc_new() {
        let lines = r#"
#$ _SPACE_= 
#$ _CUDART_=cudart
#$ _HERE_=/usr/local/cuda/bin
#$ _THERE_=/usr/local/cuda/bin
#$ _TARGET_SIZE_=
#$ _TARGET_DIR_=
#$ _TARGET_SIZE_=64
#$ TOP=/usr/local/cuda/bin/..
#$ NVVMIR_LIBRARY_DIR=/usr/local/cuda/bin/../nvvm/libdevice
#$ LD_LIBRARY_PATH=/usr/local/cuda/bin/../lib::/usr/local/cuda/lib64
#$ PATH=/usr/local/cuda/bin/../nvvm/bin:/usr/local/cuda/bin:/home/pearson/software/robo3t-1.2.1-linux-x86_64-3e50a65/bin:/home/pearson/software/toolchains/llvm-trunk/bin:/home/pearson/bin:/home/pearson/.cargo/bin:/home/pearson/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/usr/local/cuda/bin
#$ INCLUDES="-I/usr/local/cuda/bin/..//include"  
#$ LIBRARIES=  "-L/usr/local/cuda/bin/..//lib64/stubs" "-L/usr/local/cuda/bin/..//lib64"
#$ CUDAFE_FLAGS=
#$ PTXAS_FLAGS=
"#.split("\n").map(|l| l.to_owned()).collect::<Vec<_>>();

        let nvcc = Nvcc {
            path: PathBuf::from("."),
            includes: vec![],
            libraries: vec![],
        };
        let (includes, libraries) = Nvcc::parse_verbose(&lines);

        assert_eq!(
            includes,
            vec![PathBuf::from("/usr/local/cuda/bin/..//include")]
        );
        assert_eq!(
            libraries,
            vec![
                PathBuf::from("/usr/local/cuda/bin/..//lib64/stubs"),
                PathBuf::from("/usr/local/cuda/bin/..//lib64"),
            ]
        );
    }
}

extern crate find;
extern crate glob;
extern crate regex;

use regex::Regex;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::path::Path;
use std::vec::Vec;
use std::string::String;
use glob::MatchOptions;
use find::Find;

use std::io::{self, BufRead};

#[derive(Clone, Debug)]
enum ErrorKind {
    IOError,
    /// Invalid architecture supplied.
    ArchitectureInvalid,
    /// Environment variable not found, with the var in question as extra info.
    EnvVarNotFound,
    /// Error occurred while using external tools (ie: invocation of compiler).
    ToolExecError,
    /// Error occurred due to missing external tools.
    ToolNotFound,
}

/// Represents an internal error that occurred, with an explanation.
#[derive(Clone, Debug)]
pub struct Error {
    /// Describes the kind of error that occurred.
    kind: ErrorKind,
    /// More explanation of error that occurred.
    message: String,
}

impl Error {
    fn new(kind: ErrorKind, message: &str) -> Error {
        Error {
            kind: kind,
            message: message.to_owned(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::new(ErrorKind::IOError, &format!("{}", e))
    }
}

/// Backup search directory globs for FreeBSD and Linux.
const SEARCH_LINUX: &[&str] = &["/usr/local/cuda/bin", "/usr/local/cuda*/bin"];

/// Backup search directory globs for OS X.
const SEARCH_OSX: &[&str] = &[];

/// Backup search directory globs for Windows.
const SEARCH_WINDOWS: &[&str] = &[];

#[derive(Debug, Clone)]
pub struct Nvcc {
    /// where nvcc is
    path: PathBuf,
    /// nvcc's include directories
    includes: Vec<PathBuf>,
    /// nvcc's library directories
    libraries: Vec<PathBuf>,
}

impl Nvcc {
    pub fn new(path: PathBuf) -> Option<Nvcc> {
        let out = Command::new(&path).arg("-v").arg(".").output();
        let out = match out {
            Err(e) => return None,
            Ok(out) => out,
        };

        let lines = out.stderr
            .lines()
            .filter_map(|l| if let Ok(l) = l { Some(l) } else { None })
            .collect::<Vec<_>>();

        let (includes, libraries) = Nvcc::parse_verbose(&lines);

        Some(Nvcc {
            path: path,
            includes: includes,
            libraries: libraries,
        })
    }

    fn parse_verbose(lines: &[String]) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let re = Regex::new(r#""-[IL]([.[^"]]*)""#).unwrap();

        let inc_lines = lines
            .iter()
            .filter(|l| l.starts_with("#$ INCLUDES="))
            .collect::<Vec<_>>();

        let lib_lines = lines
            .iter()
            .filter(|l| l.starts_with("#$ LIBRARIES="))
            .collect::<Vec<_>>();

        let mut includes = vec![];
        let mut libraries = vec![];

        for line in inc_lines {
            // eprintln!("{:?}", line);
            includes.append(&mut re.captures_iter(line)
                .map(|c| PathBuf::from(c.get(1).unwrap().as_str()))
                .collect::<Vec<_>>())
        }

        for line in lib_lines {
            // eprintln!("{:?}", line);
            libraries.append(&mut re.captures_iter(line)
                .map(|c| PathBuf::from(c.get(1).unwrap().as_str()))
                .collect::<Vec<_>>())
        }

        (includes, libraries)
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn includes(&self) -> &Vec<PathBuf> {
        &self.includes
    }

    pub fn libraries(&self) -> &Vec<PathBuf> {
        &self.libraries
    }
}

pub struct Build {
    cargo_metadata: bool,
    compiler: Option<PathBuf>,
    files: Vec<PathBuf>,
    host: Option<String>,
    // output: String,
    out_dir: Option<PathBuf>,
    //     include_directories: Vec<PathBuf>,
    //     definitions: Vec<(String, Option<String>)>,
    //     objects: Vec<PathBuf>,
    flags: Vec<String>,
    //     flags_supported: Vec<String>,
    //     known_flag_support_status: Arc<Mutex<HashMap<String, bool>>>,
    //     files: Vec<PathBuf>,
    //     cpp: bool,
    link_cpp_stdlib: Option<Option<String>>,
    static_flag: bool,
    target: Option<String>,
    //     opt_level: Option<String>,
    //     debug: Option<bool>,
    //     env: Vec<(OsString, OsString)>,
    //     archiver: Option<PathBuf>,
    //     pic: Option<bool>,
    //     static_crt: Option<bool>,
    //     shared_flag: Option<bool>,
    //     warnings_into_errors: bool,
    // warnings: bool,
}

impl Build {
    pub fn new() -> Build {
        Build {
            cargo_metadata: true,
            compiler: None,
            files: vec![],
            flags: vec![],
            host: None,
            out_dir: None,
            target: None,
            link_cpp_stdlib: None,
            static_flag: true,
        }
    }

    pub fn file<P: AsRef<Path>>(&mut self, p: P) -> &mut Build {
        self.files.push(p.as_ref().to_path_buf());
        self
    }

    pub fn files<P>(&mut self, p: P) -> &mut Build
    where
        P: IntoIterator,
        P::Item: AsRef<Path>,
    {
        for file in p.into_iter() {
            self.file(file);
        }
        self
    }

    pub fn link_cpp_stdlib(&mut self) -> &mut Build {
        self.link_cpp_stdlib = Some(None);
        self
    }

    pub fn set_cpp_stdlib(&mut self, s: &str) -> &mut Build {
        self.link_cpp_stdlib = Some(Some(s.to_owned()));
        self
    }

    pub fn flag<P: ToString>(&mut self, p: P) -> &mut Build {
        self.flags.push(p.to_string());
        self
    }

    fn print(&self, s: &str) {
        if self.cargo_metadata {
            println!("{}", s);
        }
    }

    fn getenv(&self, v: &str) -> Option<String> {
        let r = env::var(v).ok();
        self.print(&format!("{} = {:?}", v, r));
        r
    }

    fn getenv_unwrap(&self, v: &str) -> Result<String, Error> {
        match self.getenv(v) {
            Some(s) => Ok(s),
            None => Err(Error::new(
                ErrorKind::EnvVarNotFound,
                &format!("Environment variable {} not defined.", v.to_string()),
            )),
        }
    }

    fn get_out_dir(&self) -> Result<PathBuf, Error> {
        match self.out_dir.clone() {
            Some(p) => Ok(p),
            None => Ok(env::var_os("OUT_DIR").map(PathBuf::from).ok_or_else(|| {
                Error::new(
                    ErrorKind::EnvVarNotFound,
                    "Environment variable OUT_DIR not defined.",
                )
            })?),
        }
    }

    fn get_target(&self) -> Result<String, Error> {
        match self.target.clone() {
            Some(t) => Ok(t),
            None => Ok(self.getenv_unwrap("TARGET")?),
        }
    }

    fn get_host(&self) -> Result<String, Error> {
        match self.host.clone() {
            Some(h) => Ok(h),
            None => Ok(self.getenv_unwrap("HOST")?),
        }
    }

    fn get_compiler(&self) -> Result<PathBuf, Error> {
        let host = self.get_host()?;
        let target = self.get_target()?;

        if let Some(compiler) = self.compiler.clone() {
            return Ok(compiler);
        } else if let Some(compiler) = self.getenv("COMPILER") {
            return Ok(PathBuf::from(compiler));
        } else if host == target {
            return Ok(PathBuf::from("g++"));
        } else {
            match target.as_ref() {
                "x86_64-unknown-linux-musl" => Ok(PathBuf::from("g++")),
                "powerpc64le-unknown-linux-gnu" => Ok(PathBuf::from("powerpc64le-linux-gnu-g++")),
                _ => {
                    println!("target was {}", target);
                    Err(Error::new(
                        ErrorKind::ArchitectureInvalid,
                        "couldn't find g++",
                    ))
                }
            }
        }
    }

    fn get_nvcc(&self) -> Result<Nvcc, Error> {
        let nvcc_path = Find::new("nvcc")
            .search_env("NVCC_PATH")
            .search_globs(SEARCH_LINUX)
            .execute();

        match Find::new("nvcc")
            .search_env("NVCC_PATH")
            .search_globs(SEARCH_LINUX)
            .execute()
        {
            Ok(path) => return Ok(Nvcc::new(path).unwrap()),
            Err(s) => return Err(Error::new(ErrorKind::ToolNotFound, s.as_str())),
        };
    }

    fn get_ar(&self) -> Result<String, Error> {
        let host = self.get_host()?;
        let target = self.get_target()?;

        if host == target {
            return Ok(String::from("ar"));
        } else {
            match target.as_ref() {
                "x86_64-unknown-linux-musl" => Ok(String::from("ar")),
                "powerpc64le-unknown-linux-gnu" => Ok(String::from("powerpc64le-linux-gnu-ar")),
                _ => {
                    println!("target was {}", target);
                    Err(Error::new(
                        ErrorKind::ArchitectureInvalid,
                        "couldn't find ar",
                    ))
                }
            }
        }
    }

    fn try_compile_object(&self, obj: &PathBuf, src: &PathBuf) -> Result<(), Error> {
        let compiler = self.get_compiler()?;
        let nvcc = self.get_nvcc()?;
        let incs = nvcc.includes();

        let out = Command::new("nvcc")
            .args(&self.flags)
            .arg("-ccbin")
            .arg(compiler)
            .arg("-rdc=true")
            .arg("-c")
            .arg("-Xcompiler")
            .arg("-fPIC")
            .arg("-Xcompiler")
            .args(incs)
            .arg("-o")
            .arg(obj)
            .arg(src)
            .output()
            .expect("failed to execute process");

        println!("compile:");
        println!("status: {}", out.status);
        println!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&out.stderr));

        if out.status.success() {
            return Ok(());
        } else {
            return Err(Error::new(
                ErrorKind::ToolExecError,
                "couldn't compile object",
            ));
        };
    }

    fn try_device_link(&self, output: &PathBuf, objects: &Vec<PathBuf>) -> Result<(), Error> {
        let compiler = self.get_compiler()?;
        let nvcc = self.get_nvcc()?;
        let incs = nvcc.includes();

        let out = Command::new("nvcc")
            .args(&self.flags)
            .arg("-ccbin")
            .arg(compiler)
            .arg("-dlink")
            .arg("-Xcompiler")
            .arg("-fPIC")
            .arg("-Xcompiler")
            .args(
                incs.iter()
                    .map(|i| String::from("-I") + i.to_str().unwrap()),
            )
            .arg("-o")
            .arg(output)
            .args(objects)
            .output()
            .expect("failed to execute process");

        println!("dlink:");
        println!("status: {}", out.status);
        println!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&out.stderr));

        if out.status.success() {
            return Ok(());
        } else {
            return Err(Error::new(
                ErrorKind::ToolExecError,
                "couldn't device link compiled objects",
            ));
        };
    }

    fn try_archive(&self, output: &PathBuf, objects: &Vec<PathBuf>) -> Result<(), Error> {
        let ar = self.get_ar()?;
        println!("{} {:?} {:?}", ar, output, objects);

        let out = Command::new(ar)
            .arg("-rcs")
            .arg(output)
            .args(objects)
            .output()
            .expect("failed to execute process");

        // let out = Command::new("nvcc")
        //     .arg("-lib")
        //     .arg(output)
        //     .args(objects)
        //     .output()
        //     .expect("failed to execute process");

        println!("status: {}", out.status);
        println!("stdout: {}", String::from_utf8_lossy(&out.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&out.stderr));

        if out.status.success() {
            return Ok(());
        } else {
            return Err(Error::new(
                ErrorKind::ToolExecError,
                "couldn't archive device-linked objects",
            ));
        };
    }

    pub fn try_compile(&self, output: &str) -> Result<(), Error> {
        let out_dir = self.get_out_dir()?;
        let host = self.get_host()?;
        let target = self.get_target()?;
        let nvcc = self.get_nvcc()?;

        let out_path = if self.static_flag {
            let mut out_name = String::from("lib");
            out_name.push_str(output);
            out_name.push_str(".a");
            out_dir.join(out_name)
        } else {
            let mut out_name = String::from("lib");
            out_name.push_str(output);
            out_name.push_str(".so");
            out_dir.join(out_name)
        };

        // Compile objects
        let mut objects = Vec::new();
        for file in self.files.clone() {
            let obj = out_dir.join(file.clone()).with_extension("o");
            self.try_compile_object(&obj, &file)?;
            objects.push(obj);
        }

        // Device link
        let dev_linked_obj = out_dir.join("__dlink.o");
        self.try_device_link(&dev_linked_obj, &objects)?;

        // Archive
        let mut all_objs = vec![dev_linked_obj];
        all_objs.append(&mut objects.clone());
        self.try_archive(&out_path, &all_objs)?;

        // Link against generated library
        self.print(&format!(
            "cargo:rustc-link-search=native={}",
            out_dir.to_str().unwrap()
        ));
        self.print(&format!("cargo:rustc-link-lib=static={}", output));

        // Link against cuda libs
        if host != target {
            let raw = self.getenv_unwrap("CUDA_TARGET")?;
            let cuda_lib_path = PathBuf::from(raw).join("lib64");
            self.print(&format!(
                "cargo:rustc-link-search=native={}",
                cuda_lib_path.to_str().unwrap()
            ));
        } else {
            for path in nvcc.libraries() {
                self.print(&format!(
                    "cargo:rustc-link-search=native={}",
                    path.to_str().unwrap()
                ));
            }
        };

        self.print("cargo:rustc-link-lib=cudart");
        self.print("cargo:rustc-link-lib=cudadevrt");

        // rerun if files changes
        for file in self.files.clone() {
            self.print(&format!(
                "cargo:rerun-if-changed={}",
                file.to_str().unwrap()
            ));
        }

        if let Some(ref lib) = self.link_cpp_stdlib {
            let lib = match lib {
                &Some(ref lib) => lib.as_str(),
                &None => "stdc++",
            };
            self.print(&format!("cargo:rustc-link-lib={}", lib));
        }

        Ok(())
    }

    pub fn compile(&mut self, output: &str) {
        if let Err(e) = self.try_compile(output) {
            fail(&e.message);
        }
    }
}

fn fail(s: &str) -> ! {
    panic!("\n\nInternal error occurred: {}\n\n", s)
}
