use toolshed::{Arena, BloomMap, List};

#[derive(PartialEq)]
enum ScopeKind {
    Block,
    Function,
}

struct ScopeFrame<'ast> {
    kind: ScopeKind,
    vars: HashMap<&'ast str, &'ast str>,
}

pub struct Scope<'ast> {
    arena: &'ast Arena,
    frame: usize,
    frames: Vec<ScopeFrame<'ast>>,
}

impl<'ast> Scope<'ast> {
    pub fn new(arena: &'ast Arena) -> Self {
        let mut frames = Vec::with_capacity(8);

        frames.push(ScopeFrame {
            kind: ScopeKind::Function,
            vars: HashMap::new()
        });

        Scope {
            arena,
            frame: 0,
            frames
        }
    }

    fn push(&mut self, kind: ScopeKind) {
        self.frame += 1;

        if let Some(frame) = self.frames.get_mut(self.frame) {
            frame.kind = kind;
            return;
        }

        self.frames.push(ScopeFrame {
            kind,
            vars: HashMap::new()
        });
    }

    pub fn block_frame(&mut self) {
        self.push(ScopeKind::Block)
    }

    pub fn function_frame(&mut self) {
        self.push(ScopeKind::Function)
    }

    pub fn pop(&mut self) {
        self.frames[self.frame].vars.clear();
        self.frame -= 1;
    }

    pub fn has_in_block(&self, var: &str) -> bool {
        self.frames[self.frame].vars.contains_key(var)
    }

    pub fn has_in_function(&self, var: &str) -> bool {
        for frame in self.frames[..self.frame + 1].iter().rev() {
            if frame.vars.contains_key(var) {
                return true;
            }

            if frame.kind == ScopeKind::Function {
                return false;
            }
        }

        unreachable!("Last ScopeFrame must be of Function kind")
    }

    pub fn set_in_block(&mut self, var: &'ast str) {
        self.frames[self.frame].vars.insert(var, var);
    }

    pub fn set_in_function(&mut self, var: &'ast str) {
        let frame = self.frames[..self.frame + 1]
                        .iter_mut()
                        .rev()
                        .skip_while(|frame| frame.kind != ScopeKind::Function)
                        .next()
                        .expect("Must have a Function kind scope");

        frame.vars.insert(var, var);
    }

    pub fn set_unique_in_block(&mut self, var: &'ast str) -> &'ast str {
        let frame = &mut self.frames[self.frame];

        Scope::set_unique_in_frame(&self.arena, frame, var)
    }

    pub fn set_unique_in_function(&mut self, var: &'ast str) -> &'ast str {
        let frame = self.frames[..self.frame + 1]
                        .iter_mut()
                        .rev()
                        .skip_while(|frame| frame.kind != ScopeKind::Function)
                        .next()
                        .expect("Must have a Function kind scope");

        Scope::set_unique_in_frame(&self.arena, frame, var)
    }

    fn set_unique_in_frame(arena: &'ast Arena, frame: &mut ScopeFrame<'ast>, var: &'ast str) -> &'ast str {
        if let Entry::Vacant(vacant) = frame.vars.entry(var) {
            vacant.insert(var);
            return var;
        }

        let mut attempt = 1;

        loop {
            let altered = format!("{}${}", var, attempt);

            if !frame.vars.contains_key(altered.as_str()) {
                let var = arena.alloc_string(altered);
                frame.vars.insert(var, var);
                return var;
            }

            attempt += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope() {
        let arena = Arena::new();
        let mut scope = Scope::new(&arena);

        scope.set_in_block("foo");
        scope.set_in_function("bar");

        scope.function_frame();

        scope.set_in_block("fooz");

        scope.block_frame();

        scope.set_in_block("baz");
        scope.set_in_function("qux");

        assert_eq!(scope.has_in_function("fooz"), true);
        assert_eq!(scope.has_in_function("baz"), true);
        assert_eq!(scope.has_in_function("qux"), true);

        assert_eq!(scope.has_in_block("baz"), true);
        assert_eq!(scope.has_in_block("qux"), false);
        assert_eq!(scope.has_in_block("fooz"), false);

        assert_eq!(scope.has_in_function("foo"), false);
        assert_eq!(scope.has_in_function("bar"), false);

        scope.pop();

        assert_eq!(scope.has_in_block("baz"), false);

        assert_eq!(scope.has_in_block("qux"), true);
        assert_eq!(scope.has_in_block("fooz"), true);
        assert_eq!(scope.has_in_function("qux"), true);
        assert_eq!(scope.has_in_function("fooz"), true);

        scope.pop();

        assert_eq!(scope.has_in_block("foo"), true);
        assert_eq!(scope.has_in_block("bar"), true);
        assert_eq!(scope.has_in_function("foo"), true);
        assert_eq!(scope.has_in_function("bar"), true);
    }

    #[test]
    fn set_unique_in_block() {
        let arena = Arena::new();
        let mut scope = Scope::new(&arena);

        assert_eq!(scope.set_unique_in_block("foo"), "foo");
        assert_eq!(scope.set_unique_in_block("foo"), "foo$1");
        assert_eq!(scope.set_unique_in_block("foo"), "foo$2");
        assert_eq!(scope.set_unique_in_block("foo"), "foo$3");

        assert_eq!(scope.set_unique_in_block("bar"), "bar");
        assert_eq!(scope.set_unique_in_block("bar"), "bar$1");
        assert_eq!(scope.set_unique_in_block("bar"), "bar$2");
        assert_eq!(scope.set_unique_in_block("bar"), "bar$3");

        assert_eq!(scope.has_in_block("foo"), true);
        assert_eq!(scope.has_in_block("foo$1"), true);
        assert_eq!(scope.has_in_block("foo$2"), true);
        assert_eq!(scope.has_in_block("foo$3"), true);
        assert_eq!(scope.has_in_block("foo$4"), false);
    }
}
