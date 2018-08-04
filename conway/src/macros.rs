macro_rules! string_from_file {
    ($path:expr) => {
        // let path: Path = path;
        read_to_string($path).unwrap()
    };
}
