use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn create_middleware_factory(scoped_ident: Ident, factory_ident: Ident, middleware_ident: Ident) -> TokenStream {
    (quote! {
        pub struct #factory_ident;

        impl<S, B> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for #factory_ident
            where
                S: actix_web::dev::Service<actix_web::dev::ServiceRequest, Response=actix_web::dev::ServiceResponse<B>, Error=actix_web::Error>,
                S::Future: 'static,
                B: 'static,
        {
            type Response =  actix_web::dev::ServiceResponse<B>;
            type Error = actix_web::Error;
            type Transform = #middleware_ident<S>;
            type InitError = ();
            type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

            fn new_transform(&self, service: S) -> Self::Future {
                #scoped_ident::__init_context(Default::default());
                std::future::ready(Ok(#middleware_ident { service }))
            }
        }
    }).into()
}

pub fn create_middleware(scoped_ident: Ident, middleware_error_ident: TokenStream, middleware_ident: Ident) -> TokenStream {
    (quote! {
        pub struct #middleware_ident<S> {
            service: S,
        }

        impl<S, B> actix_web::dev::Service<actix_web::dev::ServiceRequest> for #middleware_ident<S>
            where
                S: actix_web::dev::Service<actix_web::dev::ServiceRequest, Response=actix_web::dev::ServiceResponse<B>, Error=actix_web::Error>,
                S::Future: 'static,
                B: 'static,
        {
            type Response = actix_web::dev::ServiceResponse<B>;
            type Error = actix_web::Error;
            type Future = std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output=Result<actix_web::dev::ServiceResponse<B>, Self::Error>>>>;

            actix_web::dev::forward_ready!(service);

            fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
                if let Err(e) = #scoped_ident::initialize() {
                    return std::boxed::Box::pin(async move { Err(actix_web::Error::from(#middleware_error_ident::from(e))) });
                };


                let fut = self.service.call(req);

                std::boxed::Box::pin(async move {
                    let res = fut.await?;

                    #scoped_ident::clear().map_err(|e| actix_web::Error::from(#middleware_error_ident::from(e)))?;

                    Ok(res)
                })
            }
        }
    }).into()
}

pub fn create_from_request(scoped_ident: Ident, middleware_error_ident: TokenStream) -> TokenStream {
    (quote! {
        impl actix_web::FromRequest for #scoped_ident {
            type Error = #middleware_error_ident;

            type Future = std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output=Result<Self, Self::Error>>>>;

            fn from_request(
                _req: &actix_web::HttpRequest,
                _payload: &mut actix_web::dev::Payload,
            ) -> Self::Future {
                std::boxed::Box::pin(async move {
                    #scoped_ident::get_or_initialize().cloned().map_err(#middleware_error_ident::from)
                })
            }
        }
    }).into()
}

