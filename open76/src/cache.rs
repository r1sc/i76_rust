use std::{collections::{HashMap}, rc::Rc};

pub struct FileCache<'a, T> {
    content: HashMap<String, Rc<T>>,
    loader: Box<dyn FnMut(&str) -> anyhow::Result<T> + 'a>,
}

impl<'a, T> FileCache<'a, T> {
    pub fn new(loader: impl FnMut(&str) -> anyhow::Result<T> + 'a) -> Self {
        FileCache {
            content: HashMap::new(),
            loader: Box::new(loader),
        }
    }

    pub fn get(&mut self, name: &str) -> anyhow::Result<&Rc<T>> {
        if self.content.contains_key(name) {
            Ok(self.content.get(name).unwrap())
        } else {
            let a = (self.loader)(name)?;
            self.content.insert(String::from(name), Rc::new(a));
            Ok(self.content.get(name).unwrap())
        }
    }
}