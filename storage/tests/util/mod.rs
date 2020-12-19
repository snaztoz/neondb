#[macro_export]
macro_rules! path_of {
    ($p: expr) => {{
        use std::path::Path;

        let path = concat!("../", $p);
        Path::new(path)
    }};
}
