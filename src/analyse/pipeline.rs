use std::marker::PhantomData;

pub struct Pipeline<DataBase> {
    db: DataBase,
}

impl<Db> Pipeline<Db> {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn run<'a, P: Pass<Input = &'a Db>>(&'a self, pass: P) -> P::Output {
        pass.run(&self.db)
    }
}

pub trait Pass {
    type Input;
    type Output;

    fn run(&self, input: Self::Input) -> Self::Output;
}

pub trait PassThen<P: Pass>
where
    P: Sized,
    Self: Sized + Pass,
    <Self as Pass>::Output: Into<P::Input>,
{
    fn then(self, pass: P) -> ThenPass<Self, P> {
        ThenPass { a: self, b: pass }
    }
}

impl<A, B> PassThen<B> for A
where
    A: Sized + Pass,
    B: Sized + Pass,
    <A as Pass>::Output: Into<B::Input>,
{
}

pub trait PassThenExt<T, P>: PassThen<P>
where
    T: Sized,
    P: Sized + Pass,
    Self: Sized + Pass,

    <Self as Pass>::Output: Into<P::Input>,
{
    fn then_into(self, pass: T) -> ThenPass<Self, P>;
}

impl<This, T, In, Out> PassThenExt<T, FnPass<T, In, Out>> for This
where
    T: Sized + IntoFnPass<In, Out>,
    T: Fn(In) -> Out,
    In: From<This::Output>,
    This: Sized + Pass,
    <This as Pass>::Output: Into<In>,
{
    fn then_into(self, pass: T) -> ThenPass<This, FnPass<T, In, Out>> {
        ThenPass {
            a: self,
            b: pass.into_pass(),
        }
    }
}

pub struct ThenPass<A: Pass, B: Pass> {
    a: A,
    b: B,
}

impl<A, B> Pass for ThenPass<A, B>
where
    A: Pass,
    B: Pass,
    <A as Pass>::Output: Into<B::Input>,
{
    type Input = <A as Pass>::Input;
    type Output = <B as Pass>::Output;

    fn run(&self, input: Self::Input) -> Self::Output {
        self.b.run(self.a.run(input).into())
    }
}

pub struct FnPass<F: Fn(In) -> Out, In, Out>(F, PhantomData<In>, PhantomData<Out>);

impl<F: Fn(In) -> Out, In, Out> Pass for FnPass<F, In, Out> {
    type Input = In;
    type Output = Out;
    fn run(&self, input: In) -> Out {
        self.0(input)
    }
}

impl<F: Fn(In) -> Out, In, Out> From<F> for FnPass<F, In, Out> {
    fn from(f: F) -> Self {
        FnPass(f, PhantomData, PhantomData)
    }
}

pub trait IntoFnPass<In, Out>: Sized {
    #[cfg(feature = "associated_type_defaults")]
    type Input = In;
    #[cfg(feature = "associated_type_defaults")]
    type Output = Out;

    fn into_pass(self) -> FnPass<Self, In, Out>
    where
        Self: Fn(In) -> Out;
}

impl<F, In, Out> IntoFnPass<In, Out> for F
where
    F: Fn(In) -> Out,
{
    fn into_pass(self) -> FnPass<F, In, Out> {
        FnPass(self, PhantomData, PhantomData)
    }
}

impl<In, Out> Pass for fn(In) -> Out {
    type Input = In;
    type Output = Out;
    fn run(&self, input: In) -> Self::Output {
        self(input)
    }
}

impl<In, Out> Pass for &dyn Fn(In) -> Out {
    type Input = In;
    type Output = Out;
    fn run(&self, input: In) -> Self::Output {
        self(input)
    }
}

#[cfg(feature = "fn_traits")]
#[cfg(feature = "unboxed_closures")]
pub mod ext {
    use super::*;

    pub trait Pass<Input> {
        type Output;

        fn run(&self, input: Input) -> Self::Output;
    }

    // impl<F: Fn<Input>, Input> Pass<Input> for F {
    //     type Output = F::Output;
    //     fn run(&self, input: Input) -> Self::Output {
    //         self.call(input)
    //     }
    // }

    pub struct FnPass<F: Fn<In>, In>(F, PhantomData<In>);

    impl<F: Fn<Input>, Input> From<F> for FnPass<F, Input> {
        fn from(f: F) -> Self {
            Self(f, PhantomData)
        }
    }

    impl<F: Fn<Input>, Input> Pass<Input> for FnPass<F, Input> {
        type Output = F::Output;

        fn run(&self, input: Input) -> Self::Output {
            self.0.call(input)
        }
    }

    pub trait IntoFnPass<In, Out>: Sized {
        fn into_pass(self) -> FnPass<Self, In>
        where
            Self: Fn<In, Output = Out>;
    }

    impl<F: Fn<In, Output = Out>, In, Out> IntoFnPass<In, Out> for F {
        fn into_pass(self) -> FnPass<Self, In> {
            FnPass(self, PhantomData)
        }
    }
}
