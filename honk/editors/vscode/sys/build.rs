fn main() {
    ts_bindgen::d_ts_buildscript("@types/vscode/index.d.ts", "index.d.rs").unwrap();
}
