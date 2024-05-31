use crate::engine::http::http_client::HttpClient;
use crate::engine::http::http_client::HttpHeader;
use crate::engine::http::http_client::HttpParameter;

pub struct HttpClientImpl {

}

impl HttpClientImpl {
    pub fn new() -> Self {
        HttpClientImpl { }
    }
}


impl HttpClient for HttpClientImpl {
    #[doc = " Call DELETE HTTP method on the spHttpParameterecified `base_url` with `headers` and `parameters`."]
#[doc = ""]
#[doc = " The method returns a JSON response encoded as `String`."]
#[must_use]
#[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
fn delete(&self,base_url: &'str,headers:Option< &'life2[HttpHeader]> ,parameters:Option< &'life3[HttpParameter]> ,request_timeout:Option<u64> ,connection_timeout:Option<u64> ,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = String> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait {
        todo!()
    }

    #[doc = " Call GET HTTP method on the specified `base_url` with `headers` and `parameters`."]
#[doc = ""]
#[doc = " The method returns a JSON response encoded as `String`."]
#[must_use]
#[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
fn get(&'life0 self,base_url: &'life1 str,headers:Option< &'life2[HttpHeader]> ,parameters:Option< &'life3[HttpParameter]> ,request_timeout:Option<u64> ,connection_timeout:Option<u64> ,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = String> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,Self:'async_trait {
        todo!()
    }

    #[doc = " Call PATCH HTTP method on the specified `base_url` with `headers`, `parameters`, and `body`."]
#[doc = ""]
#[doc = " The `body` is a serialized JSON in `String` representation."]
#[doc = " The method returns a JSON response encoded as `String`."]
#[must_use]
#[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
fn patch(&'life0 self,base_url: &'life1 str,body:Option< &'life2 str> ,headers:Option< &'life3[HttpHeader]> ,parameters:Option< &'life4[HttpParameter]> ,request_timeout:Option<u64> ,connection_timeout:Option<u64> ,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = String> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait {
        todo!()
    }

    #[doc = " Call POST HTTP method on the specified `base_url` with `headers`, `parameters`, and `body`."]
#[doc = ""]
#[doc = " The `body` is a serialized JSON in `String` representation."]
#[doc = " The method returns a JSON response encoded as `String`."]
#[must_use]
#[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
fn post(&'life0 self,base_url: &'life1 str,body:Option< &'life2 str> ,headers:Option< &'life3[HttpHeader]> ,parameters:Option< &'life4[HttpParameter]> ,request_timeout:Option<u64> ,connection_timeout:Option<u64> ,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = String> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait {
        todo!()
    }

    #[doc = " Call PUT HTTP method on the specified `base_url` with `headers`, `parameters`, and `body`."]
#[doc = ""]
#[doc = " The `body` is a serialized JSON in `String` representation."]
#[doc = " The method returns a JSON response encoded as `String`."]
#[must_use]
#[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
fn put(&'life0 self,base_url: &'life1 str,body:Option< &'life2 str> ,headers:Option< &'life3[HttpHeader]> ,parameters:Option< &'life4[HttpParameter]> ,request_timeout:Option<u64> ,connection_timeout:Option<u64> ,) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = String> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait {
        todo!()
    }
}