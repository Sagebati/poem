use std::sync::Arc;

use crate::error::{Error, ErrorNotFound};
use crate::uri::Uri;
use crate::{Endpoint, Middleware, Request, Response, Result};

pub struct StripPrefix {
    prefix: Arc<str>,
}

impl StripPrefix {
    pub fn new(prefix: impl AsRef<str>) -> Self {
        Self {
            prefix: prefix.as_ref().into(),
        }
    }
}

impl Middleware for StripPrefix {
    fn transform<T: Endpoint>(&self, ep: T) -> Box<dyn Endpoint> {
        Box::new(StripPrefixImpl {
            inner: ep,
            prefix: self.prefix.clone(),
        })
    }
}

struct StripPrefixImpl<E> {
    inner: E,
    prefix: Arc<str>,
}

#[async_trait::async_trait]
impl<E: Endpoint> Endpoint for StripPrefixImpl<E> {
    async fn call(&self, mut req: Request) -> Result<Response> {
        let mut parts = req.uri().clone().into_parts();

        if let Some(path) = parts
            .path_and_query
            .as_ref()
            .and_then(|p| p.as_str().strip_prefix(&*self.prefix))
        {
            parts.path_and_query = Some(path.parse()?);
        } else {
            return Err(Error::not_found(ErrorNotFound));
        }

        let new_uri = Uri::from_parts(parts)?;
        req.set_uri(new_uri);
        self.inner.call(req).await
    }
}