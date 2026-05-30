use variance_family::Lend;


#[expect(missing_docs, reason = "TODO")]
#[derive(Debug, Clone, Copy)]
pub enum SelfRefCases<N, R, M> {
    NoRef(N),
    Ref(R),
    RefMut(M),
}

#[expect(missing_docs, reason = "TODO")]
pub type SelfRefSlot<'stable, 'upper, N, R, M> = SelfRefCases<
    N,
    Lend<'stable, &'upper (), R>,
    Lend<'stable, &'upper (), M>,
>;
