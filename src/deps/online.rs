use net::NetCfg;
use libra::libra_types::account_address::AccountAddress;
use crate::cli::InputNet;
use super::DependencySource;
use super::DependencySearch;
use crate::types::ModAddr;

pub struct OnlineDependencySearch<S: AsRef<str>> {
    config: NetCfg<S>,
}

impl<Q, S> DependencySearch<Q> for OnlineDependencySearch<S>
where
    Q: Into<ModAddr>,
    S: AsRef<str>,
{
    fn search(&self, module: Q) -> anyhow::Result<(DependencySource, Vec<u8>)> {
        let m: ModAddr = module.into();
        trace!("net request for {:#X}", m);
        let (addr, name) = m.split();
        net::get(&addr, name, &self.config).map(|bytes| (DependencySource::Net, bytes))
    }
}

impl<Q> Into<Box<dyn DependencySearch<Q>>> for OnlineDependencySearch<String>
where
    Q: Into<ModAddr>,
{
    fn into(self) -> Box<dyn DependencySearch<Q>> {
        Box::new(self)
    }
}

impl<Q> Into<Box<dyn DependencySearch<Q>>> for OnlineDependencySearch<&'static str>
where
    Q: Into<ModAddr>,
{
    fn into(self) -> Box<dyn DependencySearch<Q>> {
        Box::new(self)
    }
}

impl<S: AsRef<str>> OnlineDependencySearch<S> {
    pub fn new(uri: S) -> Self {
        let config = NetCfg::new(uri);
        Self { config }
    }
}
