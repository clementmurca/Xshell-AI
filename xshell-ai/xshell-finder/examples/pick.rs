use xshell_finder::open_folder_dialog;

fn main() {
    match open_folder_dialog("Choose a project folder", None) {
        Ok(Some(path)) => println!("selected: {}", path.display()),
        Ok(None) => println!("cancelled"),
        Err(e) => eprintln!("error: {e}"),
    }
}
