// Copyright 2025 Rémi BARDON
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! An [`axum`] router that can be replaced at runtime.

#![forbid(unsafe_code)]

use std::{convert::Infallible, ops::Deref as _, sync::Arc};

use arc_swap::ArcSwap;
use axum::{body::Body, routing::future::RouteFuture, Router};

#[derive(Debug, Clone, Default)]
pub struct HotSwappableRouter {
    current_router: Arc<ArcSwap<Router>>,
}

impl HotSwappableRouter {
    #[inline]
    #[must_use]
    pub fn new(router: Router) -> Self {
        Self {
            current_router: Arc::new(ArcSwap::from_pointee(router)),
        }
    }

    #[inline]
    pub fn set(&self, new_router: Router) {
        self.current_router.store(Arc::new(new_router));
    }
}

impl tower::Service<axum::http::Request<Body>> for HotSwappableRouter {
    type Response = axum::response::Response;
    type Error = Infallible;
    type Future = RouteFuture<Infallible>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: axum::http::Request<Body>) -> Self::Future {
        let current_router = Arc::clone(&self.current_router);
        let mut router: Router = current_router.load_full().deref().clone();
        router.call(req)
    }
}
