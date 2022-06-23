#![feature(negative_impls)]
#![deny(unsafe_op_in_unsafe_fn)]
use core::mem;
use core::ptr;

#[derive(Clone, Copy)]
struct IntrusiveListNode<T> {
    value: ptr::NonNull<T>,
    next: Option<ptr::NonNull<IntrusiveListNode<T>>>,
}

pub struct IntrusiveList<T> {
    head: Option<ptr::NonNull<IntrusiveListNode<T>>>,
}

impl<T> Default for IntrusiveList<T> {
    fn default() -> Self {
        Self { head: None }
    }
}

/// A helper structure to implement `Debug`, since I need to have
/// exclusive read access to it.
pub struct Dbg<'a, T>(&'a mut IntrusiveList<T>);

impl<'a, T: core::fmt::Debug + 'a> core::fmt::Debug for Dbg<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_list();
        let mut current = self.0.head;
        while let Some(curr) = current {
            builder.entry(unsafe { curr.as_ref().value.as_ref() });
            current = unsafe { curr.as_ref().next };
        }

        builder.finish()
    }
}

/// A list where you can add elements from wherever you want.
/// Due to safety reasons, the only possible thing to do with this is to read the head and to cons
/// temporally.
///
/// A direct `Debug` implementation is not possible since immutable references to this structure
/// aren't safe and I could not tell the compiler that `&IntrusiveList<T>: !Send + !Sync`. If you
/// want to debug the list use the [`debug`](IntrusiveList::debug) method.
///
/// A `Clone` implementation is not sound due to the intrusive list containing `&mut T`s
/// disguised in `ptr::NonNull<T>` to allow for multiple lifetimes to participate.
impl<T> IntrusiveList<T> {
    /// Adds the reference, runs `cont`, pops the reference.
    pub fn with_cons<O>(&mut self, value: &mut T, cont: impl FnOnce(&mut Self) -> O) -> O {
        // NOTE: no checks are needed since we're being given a *mutable reference*, which is NOT
        // copyable and MUST be moved.
        let mut new_node = IntrusiveListNode {
            // SAFE: `value` is a reference
            value: unsafe { ptr::NonNull::new_unchecked(value) },
            next: self.head.take(),
        };
        self.head = Some(unsafe { ptr::NonNull::new_unchecked(&mut new_node) });
        let result = cont(self);

        self.head = new_node.next;

        result
    }

    pub fn head(&self) -> Option<&T> {
        self.head
            .map(|node| unsafe { node.as_ref().value.as_ref() })
    }

    pub fn head_mut(&mut self) -> Option<&mut T> {
        self.head
            .map(|mut node| unsafe { node.as_mut().value.as_mut() })
    }

    /// Get an iterator to immutable references of the list values.
    /// NOTE: we can't use an immutable reference due to the possibility of iterator invalidation in
    /// multithreaded code. Even though we don't mutate the structure, `&mut` ensures that
    /// have *exclusive* access to the list, which means no iterator invalidation is possible.
    pub fn iter(&mut self) -> Iter<'_, T> {
        Iter {
            current: self.head,
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            current: self.head,
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn debug(&mut self) -> Dbg<'_, T> {
        Dbg(self)
    }
}

impl<T> !Send for IntrusiveList<T> {}
impl<T> !Sync for IntrusiveList<T> {}
// FIXME: impl<T> !Send for &IntrusiveList<T>?

pub struct Iter<'a, T> {
    current: Option<ptr::NonNull<IntrusiveListNode<T>>>,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        // SAFE: valid by list impl.
        let value_ref = unsafe { current.as_ref().value.as_ref() };
        self.current = unsafe { current.as_ref().next };
        Some(value_ref)
    }
}

impl<'a, T: 'a> core::iter::FusedIterator for Iter<'a, T> {}

pub struct IterMut<'a, T> {
    current: Option<ptr::NonNull<IntrusiveListNode<T>>>,
    _phantom: core::marker::PhantomData<&'a ()>,
}
impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.current.take()?;
        // SAFE: valid by list impl.
        let value_ref = unsafe { current.as_mut().value.as_mut() };
        self.current = unsafe { current.as_ref().next };
        Some(value_ref)
    }
}
impl<'a, T: 'a> core::iter::FusedIterator for IterMut<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_cons() {}
}
