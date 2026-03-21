use compact_str::CompactString;
use rustc_hash::FxHashMap;

#[derive(Debug,Default)]
struct Node {
    children: FxHashMap<CompactString,Node>,
}
#[derive(Debug,Default)]
pub struct PathCache {
    root_node: Node,
    root_path: CompactString,
}
impl PathCache {
    
    pub fn add(&mut self, rel_path: &str) {
        let components = rel_path.trim_matches('/').split('/').map(CompactString::from);
        
        let mut node = &mut self.root_node;
        for component in components {
            node = node.children.entry(component).or_default();
        }
    }
    
}