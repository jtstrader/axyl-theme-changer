use axyl_theme_changer as axtc;
use std::env;
use std::process::Command;

/// Reformats the Axyl Linux Distro with a provided color scheme file.
const ALC_PATH: &'static str = "/home/jtstr/.config/bspwm/alacritty/colors.yml";
const PLY_PATH: &'static str = "/home/jtstr/.config/bspwm/polybar/colors";
const BSP_PATH: &'static str = "/home/jtstr/.config/bspwm/bspwmrc";
const WLP_PATH: &'static str = "/home/jtstr/.config/bspwm/wallpapers";

fn main() {
    // attempt to load all file information
    // files to be changed:
    //   1. alacritty/colors.yml (change colors directly)
    //   2. polybar/colors (change colors directly)
    //   3. bspwmrc (use regex to change theme= line)

    // attempt to read the color information into the HashMap
    let args_list: Vec<String> = env::args().collect();
    if args_list.len() == 1 {
        panic!("no provided input file");
    }
    let color_input_file: &str = &args_list[1];

    axtc::verify_input_file(color_input_file);
    axtc::write_colors(color_input_file, ALC_PATH, PLY_PATH, BSP_PATH, WLP_PATH);
    issue_refresh();
}

/// Refresh Polybar and bspwm
fn issue_refresh() {
    Command::new("pkill")
        .arg("polybar")
        .spawn()
        .expect("failed to pkill polybar");
    Command::new("bspc")
        .args(vec!["wm", "-r"])
        .spawn()
        .expect("failed to restart bspwm");
}
