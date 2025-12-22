use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use winreg::enums::*;
use winreg::RegKey;

fn default_install_dir() -> PathBuf {
    // Windows-only installer default
    if cfg!(windows) {
        if let Some(dir) = dirs_next::data_local_dir() {
            return dir.join("Programs").join("FerrisGen");
        }
        return PathBuf::from(r"C:\Program Files\FerrisGen");
    }
    // If somehow run on non-Windows, fallback to a local folder (but we exit early)
    PathBuf::from("./ferrisgen-install")
}

fn ask(prompt: &str, default: Option<&str>) -> io::Result<String> {
    print!("{}", prompt);
    if let Some(d) = default {
        print!(" [{}]", d);
    }
    print!(": ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let s = input.trim();
    if s.is_empty() {
        Ok(default.unwrap_or("").to_string())
    } else {
        Ok(s.to_string())
    }
}

fn run_cargo_build(workspace_root: &Path, package: &str) -> io::Result<bool> {
    println!("Building {} (cargo build --release -p {})...", package, package);
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("-p")
        .arg(package)
        .current_dir(workspace_root)
        .status()?;
    Ok(status.success())
}

fn copy_binary(src: &Path, dst_dir: &Path) -> io::Result<()> {
    if !dst_dir.exists() {
        fs::create_dir_all(dst_dir)?;
    }
    let file_name = src.file_name().unwrap();
    let dst = dst_dir.join(file_name);
    fs::copy(src, &dst)?;
    // On Unix try to preserve exec bit (not used on Windows)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dst)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dst, perms)?;
    }
    Ok(())
}

fn exe_suffix() -> &'static str {
    if cfg!(windows) { ".exe" } else { "" }
}

fn add_path_to_user(install_path: &Path) -> io::Result<bool> {
    let p = install_path.to_string_lossy().to_string();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .or_else(|_| hkcu.create_subkey("Environment").map(|(k, _)| k))?;
    let cur: String = env.get_value("Path").unwrap_or_else(|_| String::new());
    // check case-insensitively
    if cur.split(';').any(|part: &str| part.eq_ignore_ascii_case(&p)) {
        return Ok(false);
    }
    let new = if cur.is_empty() { p.clone() } else { format!("{};{}", cur, p) };
    env.set_value("Path", &new)?;

    // Broadcast WM_SETTINGCHANGE to notify other apps
    let script = format!(r#"$sig='[DllImport("user32.dll",SetLastError=true)] public static extern int SendMessageTimeout(IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);'; Add-Type -MemberDefinition $sig -Name NativeMethods -Namespace Win32; $null=[Win32.NativeMethods]::SendMessageTimeout(0xffff,0x1A,0,'Environment',0,1000,[ref]([UIntPtr]0));"#);
    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .status();
    match status {
        Ok(s) if s.success() => Ok(true),
        _ => Ok(true), // path change already persisted, the broadcast failing isn't fatal
    }
}

fn create_shortcut(app_name: &str, target: &Path, args: Option<&str>, working_dir: Option<&Path>) -> io::Result<bool> {
    // Create Start Menu folder under %APPDATA%\Microsoft\Windows\Start Menu\Programs\FerrisGen
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| String::from("%APPDATA%"));
    let start_dir = PathBuf::from(appdata).join("Microsoft").join("Windows").join("Start Menu").join("Programs").join("FerrisGen");
    fs::create_dir_all(&start_dir)?;

    let shortcut_path = start_dir.join(format!("{}.lnk", app_name));
    let target_path = target.to_string_lossy().replace("'", "''");
    let working_dir_s = working_dir.map(|d| d.to_string_lossy().replace("'", "''"));
    let args_s = args.map(|s| s.replace("'", "''"));

    // Powershell script to create shortcut via WScript.Shell COM
    let mut script = format!(r#"$s = New-Object -ComObject WScript.Shell; $sc = $s.CreateShortcut('{}'); $sc.TargetPath = '{}';"#, shortcut_path.to_string_lossy(), target_path);
    if let Some(a) = &args_s {
        script.push_str(&format!(" $sc.Arguments = '{}';", a));
    }
    if let Some(wd) = &working_dir_s {
        script.push_str(&format!(" $sc.WorkingDirectory = '{}';", wd));
    }
    script.push_str(" $sc.Save();");

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .status();
    match status {
        Ok(s) if s.success() => Ok(true),
        _ => Ok(false),
    }
}

fn main() -> io::Result<()> {
    println!("FerrisGen installer (interactive, Windows-only)");

    if !cfg!(windows) {
        eprintln!("This installer currently supports Windows only.");
        return Ok(());
    }

    // default path
    let default = default_install_dir();
    let default_str = default.to_string_lossy().to_string();

    let path_input = ask("Enter install path", Some(&default_str))?;
    let install_path = if path_input.is_empty() { default } else { PathBuf::from(path_input) };

    println!("Select components to install: (A) All [default], (G) GUI only, (C) CLI only");
    let comp = ask("Type A, G or C", Some("A"))?;
    let choice = comp.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('A');

    let mut to_install = Vec::new();
    match choice {
        'G' => to_install.push("ferrisgen-gui"),
        'C' => to_install.push("ferrisgen-cli"),
        _ => {
            to_install.push("ferrisgen-cli");
            to_install.push("ferrisgen-gui");
        }
    }

    // find workspace root (two parents up from this package dir)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().and_then(|p| p.parent()).unwrap_or(&manifest_dir);

    let target_release = workspace_root.join("target").join("release");

    let mut installed_bins = Vec::new();

    for pkg in to_install.iter() {
        let bin_name = format!("{}{}", pkg.replace('-', "-"), exe_suffix());
        let src = target_release.join(&bin_name);
        if !src.exists() {
            println!("Binary '{}' not found at '{}'.", bin_name, src.display());
            let build = ask(&format!("Build {} now? (Y/n)", pkg), Some("Y"))?;
            if build.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('Y') == 'Y' {
                let ok = run_cargo_build(workspace_root, pkg)?;
                if !ok {
                    eprintln!("Failed to build {}. Aborting.", pkg);
                    return Ok(());
                }
            } else {
                eprintln!("Skipping {} because binary is missing.", pkg);
                continue;
            }
        }

        let src_after = target_release.join(&bin_name);
        if !src_after.exists() {
            eprintln!("Still missing {} after build. Aborting.", bin_name);
            return Ok(());
        }

        println!("Installing {} to {}", bin_name, install_path.display());
        copy_binary(&src_after, &install_path)?;
        installed_bins.push(src_after.file_name().unwrap().to_string_lossy().to_string());
    }

    println!("Installation complete. Installed to: {}", install_path.display());

    let add_path_resp = ask("Add install directory to user PATH? (Y/n)", Some("Y"))?;
    if add_path_resp.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('Y') == 'Y' {
        match add_path_to_user(&install_path) {
            Ok(true) => println!("Added install path to user PATH."),
            Ok(false) => println!("Install path already in PATH."),
            Err(e) => eprintln!("Failed to add to PATH: {}", e),
        }
    }

    let shortcut_resp = ask("Create Start Menu shortcuts? (Y/n)", Some("Y"))?;
    if shortcut_resp.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('Y') == 'Y' {
        // Create shortcuts for installed bins
        for bin in installed_bins.iter() {
            let exe = install_path.join(bin);
            let name = if bin.to_lowercase().contains("gui") { "FerrisGen GUI" } else { "FerrisGen CLI" };
            let (target, args, working) = if name == "FerrisGen CLI" {
                // make the shortcut launch PowerShell and run the CLI (keeps window open)
                // Target will be powershell.exe, arguments to run the program
                let pwsh = std::env::var("WINDIR").map(|w| format!("{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe", w)).unwrap_or_else(|_| String::from("powershell.exe"));
                let arg = format!("-NoExit -Command & '{}'", exe.to_string_lossy().replace("'", "''"));
                (PathBuf::from(pwsh), Some(arg), exe.parent().map(|p| p.to_path_buf()))
            } else {
                (exe.clone(), None, exe.parent().map(|p| p.to_path_buf()))
            };

            let working_dir_opt = working.as_deref();
            let args_opt = args.as_deref();
            let ok = create_shortcut(name, &target, args_opt, working_dir_opt)?;
            if ok { println!("Created Start Menu shortcut for {}", name); } else { eprintln!("Failed to create shortcut for {}", name); }
        }
    }

    println!("Done.");
    Ok(())
}
