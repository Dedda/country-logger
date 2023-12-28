use std::collections::HashMap;
use iced::widget::image::Handle;
use include_dir::{Dir, include_dir};
use lazy_static::lazy_static;

static FLAG_FILES: Dir = include_dir!("src/assets/flags");

lazy_static! {
        pub static ref FLAGS: HashMap<String, Handle> = {
            let mut map = HashMap::new();
            for file in FLAG_FILES.files() {
                let name = file.path().file_name().unwrap().to_str().unwrap()[0..2].to_uppercase();
                let contents = file.contents();
                let image = Handle::from_memory(contents);
                map.insert(name, image);
            }
            map
    };
}

