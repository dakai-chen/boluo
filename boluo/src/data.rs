use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy)]
pub struct Json<T>(pub T);

impl<T> Deref for Json<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Json<T> {
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Form<T>(pub T);

impl<T> Deref for Form<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Form<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Form<T> {
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Extension<T>(pub T);

impl<T> Deref for Extension<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Extension<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Extension<T> {
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}
