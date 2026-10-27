use std::env;

fn main() {
    // TODO: find out if liblsl already present on system and usable (if so, link to that instead)
    // println!("cargo:warning={}", "rebuilding...");
    build_liblsl();
}

// Build the liblsl library from source using cmake
fn build_liblsl() {
    let target = env::var("TARGET").unwrap();
    
    // build with cmake
    let mut cfg = cmake::Config::new("liblsl");
    cfg
        .define("LSL_NO_FANCY_LIBNAME", "ON")
        .define("LSL_BUILD_STATIC", "ON");
    if target.contains("msvc") {
        // override some C/CXX flags that the cmake crate splices in on Windows
        // (these cause the build to fail)...
        // * /EHsc sets the correct exception handling mode
        // * /GR enables RTTI
        // * /MD links in the msvcrt as a DLL, /MT statically; this follows the
        //   Rust side (`-C target-feature=+crt-static`), as the two must match
        let crt_static = env::var("CARGO_CFG_TARGET_FEATURE")
            .map_or(false, |features| features.split(',').any(|f| f == "crt-static"));
        let cxx_args = if crt_static {
            " /nologo /EHsc /MT /GR"
        } else {
            " /nologo /EHsc /MD /GR"
        };
        cfg 
            .define("WIN32", "1")
            .define("_WINDOWS", "1")
            .define("CMAKE_C_FLAGS", cxx_args)
            .define("CMAKE_CXX_FLAGS", cxx_args)
            .define("CMAKE_C_FLAGS_DEBUG", cxx_args)
            .define("CMAKE_CXX_FLAGS_DEBUG", cxx_args)
            .define("CMAKE_C_FLAGS_RELEASE", cxx_args)
            .define("CMAKE_CXX_FLAGS_RELEASE", cxx_args);
        // the cmake crate picks these for opt-level "s"/"z" and for release
        // with debug info; their defaults bring their own /MD, so keep only
        // their optimisation flags
        let min_size_rel = format!("{} /O1 /Ob1 /DNDEBUG", cxx_args);
        let rel_with_deb_info = format!("{} /Zi /O2 /Ob1 /DNDEBUG", cxx_args);
        cfg
            .define("CMAKE_C_FLAGS_MINSIZEREL", &min_size_rel)
            .define("CMAKE_CXX_FLAGS_MINSIZEREL", &min_size_rel)
            .define("CMAKE_C_FLAGS_RELWITHDEBINFO", &rel_with_deb_info)
            .define("CMAKE_CXX_FLAGS_RELWITHDEBINFO", &rel_with_deb_info);
    }
    let install_dir = cfg.build();

    // emit link directives
    let libdir = install_dir.join("lib");
    let libname = "lsl";
    println!(
        "cargo:rustc-link-search=native={}",
        libdir.to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static={}", libname);

    // make sure we also link some additional libs
    if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target.contains("windows") {
        // TODO: this is a shortcoming in the current cmake file, which should be       
        //       linking in this library (once this is fixed, we should remove this)
        println!("cargo:rustc-link-lib=dylib=bcrypt");
    } else {
        println!("cargo:rustc-link-lib=dylib=c++");
    }
}
