#[macro_export]
macro_rules! build {
    (
        #[expressions]
        $($ename:ident => $etype:ident;)*

        #[statements]
        $($sname:ident => $stype:ident;)*
    ) => (
        use $crate::ast::Name;

        pub trait Visitor<'ast> {
            type Context;

            #[inline]
            fn on_class<N: Name<'ast>>(Class<'ast, N>, &mut Self::Context) {}

            #[inline]
            fn on_function<N: Name<'ast>>(Function<'ast, N>, &mut Self::Context) {}

            // Construct methods for expressions
            $(
                #[inline]
                fn $ename(&$etype<'ast>, &ExpressionPtr<'ast>, &mut Self::Context) {}
            )*

            // Construct methods for statements
            $(
                #[inline]
                fn $sname(&$stype<'ast>, &StatementPtr<'ast>, &mut Self::Context) {}
            )*
        }

        impl<'ast, A, B, CTX> Visitor<'ast> for (A, B) where
            A: Visitor<'ast, Context = CTX>,
            B: Visitor<'ast, Context = CTX>,
        {
            type Context = CTX;

            #[inline]
            fn on_class<N: Name<'ast>>(node: Class<'ast, N>, ctx: &mut CTX) {
                A::on_class(node, ctx);
                B::on_class(node, ctx);
            }

            #[inline]
            fn on_function<N: Name<'ast>>(node: Function<'ast, N>, ctx: &mut CTX) {
                A::on_function(node, ctx);
                B::on_function(node, ctx);
            }

            // Construct methods for expressions
            $(
                #[inline]
                fn $ename(node: &$etype<'ast>, ptr: &ExpressionPtr<'ast>, ctx: &mut CTX) {
                    A::$ename(node, ptr, ctx);
                    B::$ename(node, ptr, ctx);
                }
            )*

            // Construct methods for statements
            $(
                #[inline]
                fn $sname(node: &$stype<'ast>, ptr: &StatementPtr<'ast>, ctx: &mut CTX) {
                    A::$sname(node, ptr, ctx);
                    B::$sname(node, ptr, ctx);
                }
            )*
        }
    )
}
