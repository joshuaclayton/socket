pub struct Builder<T, E> {
    values: Vec<T>,
    errors: Vec<E>,
}

impl<T, E> Into<Result<Vec<T>, Vec<E>>> for Builder<T, E> {
    fn into(self) -> Result<Vec<T>, Vec<E>> {
        if self.errors.is_empty() {
            Ok(self.values)
        } else {
            Err(self.errors)
        }
    }
}

impl<T, E> From<Vec<Result<T, E>>> for Builder<T, E> {
    fn from(records: Vec<Result<T, E>>) -> Self {
        let mut values = vec![];
        let mut errors = vec![];

        for item in records.into_iter() {
            match item {
                Ok(v) => values.push(v),
                Err(e) => errors.push(e),
            }
        }

        Builder { values, errors }
    }
}

impl<T, E> std::default::Default for Builder<T, E> {
    fn default() -> Self {
        Builder {
            values: vec![],
            errors: vec![],
        }
    }
}

impl<T, E> Builder<T, E> {
    pub fn append(&mut self, value: T) {
        self.values.push(value)
    }

    pub fn warn(&mut self, value: E) {
        self.errors.push(value)
    }

    pub fn result(&self) -> &[T] {
        &self.values
    }

    pub fn errors(&self) -> &[E] {
        &self.errors
    }

    pub fn map<U, F>(self, f: F) -> Builder<U, E>
    where
        F: Fn(T) -> U,
    {
        Builder {
            values: self.values.into_iter().map(f).collect(),
            errors: self.errors,
        }
    }
}
