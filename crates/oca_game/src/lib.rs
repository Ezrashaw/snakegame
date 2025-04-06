// macro_rules! declare_game {
//     (width: $width:literal, height: $height:literal) => {
//         // The macro will expand into the contents of this block.
//         println!("Hello!")
//     };
// }

#[macro_export]
macro_rules! import_pansi {
    ($($(#[$meta:meta])* $vis:vis const $ident:ident = pansi $file:literal;)+) => {
        $($(#[$meta])* $vis const $ident: &str = include_str!(concat!(env!("OUT_DIR"), "/", $file));)+
    };
}
