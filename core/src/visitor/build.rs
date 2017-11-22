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
            fn on_class<N: Name<'ast>>(&self, &Class<'ast, N>, &mut Self::Context) {}

            #[inline]
            fn on_function<N: Name<'ast>>(&self, &Function<'ast, N>, &mut Self::Context) {}

            #[inline]
            fn on_this(&self, &ExpressionPtr<'ast>, &mut Self::Context) {}

            // Construct methods for expressions
            $(
                #[inline]
                fn $ename(&self, &$etype<'ast>, &ExpressionPtr<'ast>, &mut Self::Context) {}
            )*

            // Construct methods for statements
            $(
                #[inline]
                fn $sname(&self, &$stype<'ast>, &StatementPtr<'ast>, &mut Self::Context) {}
            )*
        }

        impl<'ast, A, B, CTX> Visitor<'ast> for (A, B) where
            A: Visitor<'ast, Context = CTX>,
            B: Visitor<'ast, Context = CTX>,
        {
            type Context = CTX;

            #[inline]
            fn on_class<N: Name<'ast>>(&self, node: &Class<'ast, N>, ctx: &mut CTX) {
                A::on_class(&self.0, node, ctx);
                B::on_class(&self.1, node, ctx);
            }

            #[inline]
            fn on_function<N: Name<'ast>>(&self, node: &Function<'ast, N>, ctx: &mut CTX) {
                A::on_function(&self.0, node, ctx);
                B::on_function(&self.1, node, ctx);
            }

            #[inline]
            fn on_this(&self, ptr: &ExpressionPtr<'ast>, ctx: &mut CTX) {
                A::on_this(&self.0, ptr, ctx);
                B::on_this(&self.1, ptr, ctx);
            }

            // Construct methods for expressions
            $(
                #[inline]
                fn $ename(&self, node: &$etype<'ast>, ptr: &ExpressionPtr<'ast>, ctx: &mut CTX) {
                    A::$ename(&self.0, node, ptr, ctx);
                    B::$ename(&self.1, node, ptr, ctx);
                }
            )*

            // Construct methods for statements
            $(
                #[inline]
                fn $sname(&self, node: &$stype<'ast>, ptr: &StatementPtr<'ast>, ctx: &mut CTX) {
                    A::$sname(&self.0, node, ptr, ctx);
                    B::$sname(&self.1, node, ptr, ctx);
                }
            )*
        }
    )
}
