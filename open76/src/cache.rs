use std::{collections::HashMap, rc::Rc};

pub struct FileCache<'a, T> {
    content: HashMap<String, Rc<T>>,
    loader: Box<dyn FnMut(&str) -> Option<T> + 'a>,
}

impl<'a, T> FileCache<'a, T> {
    pub fn new(loader: impl FnMut(&str) -> Option<T> + 'a) -> Self {
        FileCache {
            content: HashMap::new(),
            loader: Box::new(loader),
        }
    }

    pub fn get(&mut self, name: &str) -> Option<&Rc<T>> {
        if self.content.contains_key(name) {
            self.content.get(name)
        } else {
            let a = (self.loader)(name)?;
            self.content.insert(String::from(name), Rc::new(a));
            self.content.get(name)
        }
    }
}
