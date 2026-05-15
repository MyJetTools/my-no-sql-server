fn main() {
    ci_utils::css::CssCompiler::new("./css")
        .add_file("01-styled.css")
        .add_file("02-app.css")
        .compile("./public/assets/app.css");
}
