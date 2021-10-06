use std::process::{Command, Stdio};

pub fn download( source:&str, destination:&str ) -> Result<(),String> {
    let ret = Command::new("python")
    .args(["pysrc/main.py","download",source,destination])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .output()
    .expect("python command couldn't be executed");

    match ret.status.code() {
        Some(0) => Ok(()),
        Some(x) => Err(format!("python exited with status code: {}", x)),
        None    => Err("python terminated by signal".to_string()),
    }
}

pub fn upload( source:&str, destination:&str, content_type:&str ) -> Result<(),String> {
    let ret = Command::new("python")
    .args(["pysrc/main.py","upload",source,destination,"--content-type",content_type])
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .output()
    .expect("python command couldn't be executed");

    match ret.status.code() {
        Some(0) => Ok(()),
        Some(x) => Err(format!("python exited with status code: {}", x)),
        None    => Err("python terminated by signal".to_string()),
    }
}
