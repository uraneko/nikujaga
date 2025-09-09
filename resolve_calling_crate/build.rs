fn main() {
    // is channel nightly or not
    if {
        match std::process::Command::new("rustc")
            .args(["--version"])
            .output()
        {
            Err(e) => panic!("oohohohoho"),
            Ok(ver) => (String::from_utf8_lossy(&ver.stdout)).contains("nightly"),
        }
    } {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}
