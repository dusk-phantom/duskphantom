use std::fs::read_dir;

#[allow(unused)]
pub fn edit_files_in(folder: &str, func: fn(String) -> String) {
    let paths = read_dir(folder).unwrap();
    for path in paths {
        // Get path
        let path = path.unwrap().path();

        // If path is a file, edit its contents with func
        if path.is_file() {
            let file_content = std::fs::read_to_string(&path).unwrap();
            let new_content = func(file_content);
            std::fs::write(&path, new_content).unwrap();
            return;
        }

        // If path is a directory, recurse into it
        if path.is_dir() {
            edit_files_in(path.to_str().unwrap(), func);
        }
    }
}

#[allow(unused)]
pub fn edit_files(paths: Vec<String>, func: fn(String) -> String) {
    for path in paths {
        // Get path
        let path = std::path::Path::new(&path);

        // If path is a file, edit its contents with func
        if path.is_file() {
            let file_content = std::fs::read_to_string(path).unwrap();
            let new_content = func(file_content);
            std::fs::write(path, new_content).unwrap();
            return;
        }

        // If path is a directory, recurse into it
        if path.is_dir() {
            edit_files_in(path.to_str().unwrap(), func);
        }
    }
}
