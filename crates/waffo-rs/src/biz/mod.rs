//! Business resources: domain models (types + parameters) and endpoint
//! functions. Each domain submodule declares its `Endpoint`s and exposes free
//! functions of the form `fn op(client, params, opts) -> Result<Data>`.

pub mod merchant;
pub mod order;
pub mod refund;
pub mod subscription;
