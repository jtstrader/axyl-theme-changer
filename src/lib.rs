use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

/// Data structure for maintaining all colors
#[derive(Deserialize, Debug)]
pub struct ColorScheme<'a> {
    theme: &'a str,
    color: Vec<&'a str>,
    background: &'a str,
    foreground: &'a str,
}

/// Try to open the provided file. If it cannot be opened, create the file
/// and return a File object that points to the newly created file.
pub fn verify_output_file(path: &str) {
    // If path does not exist, attempt to create the file.
    // If file creation fails, exit the program and log an error.
    let ppath: &Path = Path::new(path);
    if !ppath.exists() {
        match File::create(path) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
    } else if ppath.is_dir() {
        panic!(
            "designated output file with path \"{}\" is a directory",
            path
        );
    }
}

/// Attempt to open the passed in file for changing the color scheme.
pub fn verify_input_file(path: &str) {
    // If path does not exist, panic!
    let ppath: &Path = Path::new(path);
    if !ppath.exists() {
        panic!("provided file \"{}\" does not exist", path);
    } else if ppath.is_dir() {
        panic!("provided input file \"{}\" is a directory", path);
    }
}

/// Write out color information to the files provided
pub fn write_colors(path: &str, alc_path: &str, ply_path: &str, bsp_path: &str, wlp_path: &str) {
    // Deserialize data into our ColorScheme struct
    let data = &fs::read_to_string(path).unwrap();
    let color_scheme = match serde_json::from_str::<ColorScheme>(data) {
        Ok(cs) => {
            if cs.color.len() != 16 {
                panic!(
                    "\"{}\" contains invalid JSON, length of color array is {}, expected {}",
                    path,
                    cs.color.len(),
                    16
                );
            }
            cs
        }
        Err(e) => {
            panic!("{}", e);
        }
    };

    write_alacritty(&color_scheme, alc_path);
    write_polybar(&color_scheme, ply_path);
    write_bspwmrc(color_scheme.theme, bsp_path, wlp_path);
}

/// Write out color scheme in Alacritty format
fn write_alacritty(cs: &ColorScheme, path: &str) {
    // Open file w/ create (because we will use overwrite mode)
    match File::create(path) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    };

    let f = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();
    let mut f = BufWriter::new(f);

    let colors: [&str; 8] = [
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    ];

    // remove # from front of color code and add 0x
    let alcfmt = |s: &str| format!("0x{}", &s[1..]);

    // do not use the variable in the current scope of the fuction, as that will give the closure
    // the access to the single, mutable reference that's allowed. We instead can pass in the mutable
    // reference, treating this like a function. Only defined as a closure as its purpose is solely
    // for this function, and will be used nowhere else.
    let write_colors = |fx: &mut BufWriter<File>, bright: bool| {
        let shift = if bright { 8 } else { 0 };
        for (i, color) in colors.iter().enumerate() {
            let line = format!("    {:<10}'{}'", format!("{}:", color), cs.color[i + shift]);
            writeln!(fx, "{}", line).unwrap();
        }
        writeln!(fx).unwrap();
    };

    // header through primary colors
    writeln!(f, "# Colors ({} Theme)\n colors:", cs.theme).unwrap();
    writeln!(f, "  # Default colors").unwrap();
    writeln!(f, "  primary:").unwrap();
    writeln!(f, "    background: '{}'", alcfmt(cs.background)).unwrap();
    writeln!(f, "    foreground: '{}'", alcfmt(cs.foreground)).unwrap();
    writeln!(f).unwrap();

    // normal colors
    writeln!(f, "  # Normal colors\n  normal:").unwrap();
    write_colors(&mut f, false);

    // bright colors
    writeln!(f, "  # Bright colors\n  bright:").unwrap();
    write_colors(&mut f, true);
}

/// Write out color scheme in Polybar format
fn write_polybar(cs: &ColorScheme, path: &str) {
    // Open file w/ create (because we will use overwrite mode)
    match File::create(path) {
        Ok(_) => {}
        Err(e) => {
            panic!("{}", e);
        }
    };

    let f = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .unwrap();
    let mut f = BufWriter::new(f);

    let colors: [&str; 8] = [
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    ];

    // do not use the variable in the current scope of the fuction, as that will give the closure
    // the access to the single, mutable reference that's allowed. We instead can pass in the mutable
    // reference, treating this like a function. Only defined as a closure as its purpose is solely
    // for this function, and will be used nowhere else.
    let write_colors = |fx: &mut BufWriter<File>, bright: bool| {
        let shift = if bright { 8 } else { 0 };
        for (i, color) in colors.iter().enumerate() {
            writeln!(
                fx,
                "{} = {}",
                format!("{}{}", if bright { "alt" } else { "" }, color),
                cs.color[i + shift]
            )
            .unwrap();
        }
    };

    // write color tag and background/foreground
    writeln!(f, "[color]").unwrap();
    writeln!(f, "background = {}", cs.background).unwrap();
    writeln!(f, "foreground = {}", cs.foreground).unwrap();

    // normal colors
    write_colors(&mut f, false);

    // bright colors
    write_colors(&mut f, true);
}

/// Change the theme value saved in bspwmrc
fn write_bspwmrc(theme: &str, path: &str, wlp_path: &str) {
    // do not use File::create, as that would overrite the file and we wish to read from it first
    // confirm that the wallpaper directory is valid, otherwise inform the user that the wallpaper
    // change may not work properly, and may cause errors to be thrown
    if !Path::new(wlp_path).exists() || !Path::new(wlp_path).is_dir() {
        eprintln!("axtc: WARNING: wallpaper change not in affect, could not find wallpaper directory \"{}\"", wlp_path);
        return;
    }

    verify_output_file(path);
    let bspwmrc_contents = fs::read_to_string(path).unwrap();
    let re = Regex::new(r"theme=.*").unwrap();
    let bspwmrc_contents = re.replace(&bspwmrc_contents, format!("theme={}", theme));
    fs::write(path, &*bspwmrc_contents).unwrap();
}
