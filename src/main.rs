use std::path::{self, PathBuf};
mod config;
use slint::ComponentHandle;

slint::include_modules!();

// #[derive(Debug)]
// enum Language {
//     Go,
//     Rust,
// }

// #[derive(Debug)]
// enum Environment {
//     Linux,
//     Windows,
// }

// #[allow(non_camel_case_types)]
// #[derive(Debug)]
// enum Architecture {
//     ARM,
//     x86_64,
// }

fn main() -> Result<(), slint::PlatformError> {

    // Disable gettext on macOS due to https://github.com/Koka/gettext-rs/issues/114
    #[cfg(not(target_os = "macos"))]
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/"));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut cfg = config::AppConfig::new().expect("Failed to load config");
    let ui = AppWindow::new()?;
    // let ui_clone = ui.clone_strong();
    ui.set_amd64_cc_path(cfg.amd64.cc_path.clone().into());
    ui.set_amd64_cxx_path(cfg.amd64.cxx_path.clone().into());
    ui.set_arm_cc_path(cfg.arm.cc_path.clone().into());
    ui.set_arm_cxx_path(cfg.arm.cxx_path.clone().into());

    ui.on_show_open_dialog(move || {
        let path = show_open_dialog();
        // ui_clone.set_project_path(path.to_str().unwrap().into());

        // let mut files = path.read_dir().unwrap();
        // while let Some(file) = files.next() {
        //     let file = file.unwrap();
        //     let file_name = file.file_name();
        //     let file_name = file_name.to_str().unwrap();
        //     if file_name.ends_with(".go") {

        //     } else {

        //     }
        // }

        path.to_str().unwrap().into()
    });
    ui.on_show_open_file(move |t| {
        let path = show_open_file();

        match t.as_str() {
            "amd64_cc" => cfg.amd64.cc_path = path.to_str().unwrap().into(),
            "amd64_cxx" => cfg.amd64.cxx_path = path.to_str().unwrap().into(),
            "arm_cc" => cfg.arm.cc_path = path.to_str().unwrap().into(),
            "arm_cxx" => cfg.arm.cxx_path = path.to_str().unwrap().into(),
            _ => panic!("Unsupported type: {}", t), // 或者处理不支持的类型
        }
        cfg.save();
        path.to_str().unwrap().into()
    });
    // let (tx, rx) = std::sync::mpsc::channel();
    // let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    // let tx = Arc::new(tx);
    ui.on_compile({
        let ui_handle = ui.as_weak();
        move |c| {
            if c.language == "Go" {
                std::env::set_var("CGO_ENABLED", if c.cgo { "1" } else { "0" });
                let ui_clone = ui_handle.clone();
                match (c.os.as_str(), c.architecture.as_str()) {
                    ("Windows", "amd64") => {
                        std::env::set_var("GOOS", "windows");
                        std::env::set_var("GOARCH", "amd64");
                    }
                    ("Windows", "arm") => {
                        std::env::set_var("GOOS", "windows");
                        std::env::set_var("GOARCH", "arm64");
                    }
                    ("Linux", "amd64") => {
                        std::env::set_var("GOOS", "linux");
                        std::env::set_var("GOARCH", "amd64");
                        std::env::set_var(
                            "CC",
                            ui_clone.upgrade().unwrap().get_amd64_cc_path().as_str(),
                        );
                        std::env::set_var(
                            "CXX",
                            ui_clone.upgrade().unwrap().get_amd64_cxx_path().as_str(),
                        );
                    }
                    ("Linux", "arm") => {
                        std::env::set_var("GOOS", "linux");
                        std::env::set_var("GOARCH", "arm64");
                        std::env::set_var(
                            "CC",
                            ui_clone.upgrade().unwrap().get_arm_cc_path().as_str(),
                        );
                        std::env::set_var(
                            "CXX",
                            ui_clone.upgrade().unwrap().get_arm_cxx_path().as_str(),
                        );
                    }
                    ("MacOS", "amd64") | ("MacOS", "arm") | ("MacOS", "arm64") => {
                        std::env::set_var("GOOS", "darwin");
                        std::env::set_var(
                            "GOARCH",
                            if c.architecture == "arm" || c.architecture == "arm64" {
                                "arm64"
                            } else {
                                "amd64"
                            },
                        );
                    }
                    _ => {
                        println!("Unsupported configuration: {}-{}", c.os, c.architecture);
                        ui_clone.upgrade().unwrap().set_compile_message(
                            format!("Unsupported configuration: {}-{}", c.os, c.architecture)
                                .into(),
                        )
                    }
                }

                runtime.spawn(async move {
                    let mut cmd = std::process::Command::new("go");
                    cmd.current_dir(path::Path::new(c.project_path.as_str()));
                    cmd.arg("build");
                    cmd.arg("-ldflags=-s -w");
                    cmd.arg(".");
                    ui_clone
                        .upgrade_in_event_loop(|ui| {
                            // ui.set_compile_message("Compiling".into());
                            ui.set_compile_status(CompileStatus::Compiling);
                        })
                        .unwrap();
                    // cmd.arg("-s -w");
                    let exit_code = cmd.status().unwrap_or_default();
                    println!("{}", exit_code);

                    if !exit_code.success() {
                        println!("Build failed");
                        // ui_clone
                        //     .upgrade()
                        //     .unwrap()
                        //     .set_compile_message("CompileFailed".into());
                        ui_clone
                            .upgrade_in_event_loop(|ui| {
                                ui.set_compile_status(CompileStatus::CompileFailed);
                            })
                            .unwrap();
                    } else {
                        // ui_clone
                        //     .upgrade_in_event_loop(|ui| {
                        //         ui.set_compile_message("CompileSuccess".into());
                        //     })
                        //     .unwrap()
                        ui_clone
                            .upgrade_in_event_loop(|ui| {
                                ui.set_compile_status(CompileStatus::CompileSuccess);
                            })
                            .unwrap();
                    }
                });
            } else {
                ui_handle
                    // .clone()
                    .upgrade()
                    .unwrap()
                    .set_compile_message("NotSupport".into());
                ui_handle
                    .clone()
                    .upgrade()
                    .unwrap()
                    .set_compile_status(CompileStatus::NotSupport);
            }
        }
    });

    ui.run()
}

fn show_open_dialog() -> PathBuf {
    let mut dialog = rfd::FileDialog::new();
    dialog = dialog.set_title("Select a folder");

    dialog.pick_folder().unwrap_or_default()
}

fn show_open_file() -> PathBuf {
    let mut dialog = rfd::FileDialog::new();
    dialog = dialog.set_title("Select a file");

    dialog.pick_file().unwrap_or_default()
}
