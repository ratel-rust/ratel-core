#[macro_export]
macro_rules! build { ( $($name:ident => $type:ty;)* ) => (
    /// Helper macro for extracting Visitable::Parent type for any T: Visitable
    macro_rules! parent { ($t:ty) => (<$t as Visitable<'ast>>::Parent) }

    pub trait Visitor<'ast> {
        type Context;

        // Construct methods
        $(
            #[inline]
            fn $name(&self, &$type, &parent!($type), &mut Self::Context) {}
        )*

        fn register(&self, &mut DynamicVisitor<'ast, Self::Context>);
    }

    pub trait StaticVisitor<'ast> {
        type Context;

        // Construct associated functions
        $(
            #[inline]
            fn $name(&$type, &parent!($type), &mut Self::Context) {}
        )*

        fn register(&mut DynamicVisitor<'ast, Self::Context>);
    }

    impl<'ast, SV> Visitor<'ast> for SV
        where SV: StaticVisitor<'ast>
    {
        type Context = SV::Context;

        // Construct methods
        $(
            #[inline]
            fn $name(&self, node: &$type, ptr: &parent!($type), ctx: &mut Self::Context) {
                SV::$name(node, ptr, ctx);
            }
        )*

        #[inline]
        fn register(&self, dv: &mut DynamicVisitor<'ast, Self::Context>) {
            SV::register(dv)
        }
    }

    pub struct DynamicVisitor<'ast, CTX> {
        // Construct vectors for handlers
        $(
            pub $name: Vec<fn(&$type, &parent!($type), &mut CTX)>,
        )*
    }

    impl<'ast, CTX> DynamicVisitor<'ast, CTX> {
        pub fn new() -> Box<Self> {
            Box::new(DynamicVisitor {
                $(
                    $name: Vec::new(),
                )*
            })
        }
    }

    impl<'ast, CTX> Visitor<'ast> for DynamicVisitor<'ast, CTX> {
        type Context = CTX;

        // Construct methods
        $(
            #[inline]
            fn $name(&self, node: &$type, ptr: &parent!($type), ctx: &mut Self::Context) {
                for handler in &self.$name {
                    handler(node, ptr, ctx);
                }
            }
        )*

        fn register(&self, dv: &mut DynamicVisitor<'ast, Self::Context>) {
            $(
                dv.$name.extend_from_slice(&self.$name);
            )*
        }
    }

    impl<'ast, A, B, CTX> StaticVisitor<'ast> for (A, B) where
        A: StaticVisitor<'ast, Context = CTX>,
        B: StaticVisitor<'ast, Context = CTX>,
    {
        type Context = CTX;

        // Construct associated functions
        $(
            #[inline]
            fn $name(node: &$type, ptr: &parent!($type), ctx: &mut CTX) {
                A::$name(node, ptr, ctx);
                B::$name(node, ptr, ctx);
            }
        )*

        #[inline]
        fn register(dv: &mut DynamicVisitor<'ast, Self::Context>) {
            A::register(dv);
            B::register(dv);
        }
    }
)}
