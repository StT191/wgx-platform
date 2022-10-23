
use crate::error::*;
use ::icon_loader::{IconLoader, ThemeNameProvider, SearchPaths};
use resvg::{usvg::{Tree, Options, FitTo}, tiny_skia::{Pixmap, Transform}};


pub fn load_icon(name: &str, (w, h): (u32, u32)) -> Res<Vec<u8>> {

  let mut loader = IconLoader::new();
  loader.set_search_paths(SearchPaths::System);
  loader.set_theme_name_provider(ThemeNameProvider::KDE);
  loader.update_theme_name().convert()?;

  let icon = loader.load_icon(name).convert()?;

  let path = icon.file_for_size(w as u16).path();
  // println!("{}", path.to_str().unwrap());


  let svg_data = std::fs::read(path).convert()?;
  let rtree = Tree::from_data(&svg_data, &Options::default().to_ref()).convert()?;

  let mut pixmap = Pixmap::new(w, h).ok_or("couldn't create pixmap")?;

  resvg::render(&rtree, FitTo::Size(w, h), Transform::default(), pixmap.as_mut()).ok_or("couldn't render svg")?;

  Ok(pixmap.take())
}
