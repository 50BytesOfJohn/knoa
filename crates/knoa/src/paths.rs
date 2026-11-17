//! Where Knoa keeps its files, as a GPUI global. `knoa-paths` decides where.

use gpui::Global;
use knoa_paths::Paths;

pub struct AppPaths(pub Paths);

impl Global for AppPaths {}
