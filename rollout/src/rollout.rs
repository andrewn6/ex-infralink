#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RolloutStrategy {
    #[prost(enumeration = "rollout_strategy::StrategyType", tag = "1")]
    pub r#type: i32,
    #[prost(oneof = "rollout_strategy::Rollout", tags = "2, 3, 4")]
    pub rollout: ::core::option::Option<rollout_strategy::Rollout>,
}
/// Nested message and enum types in `RolloutStrategy`.
pub mod rollout_strategy {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum StrategyType {
        BlueGreen = 0,
        Linear = 1,
        Exponential = 2,
        Custom = 3,
    }
    impl StrategyType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                StrategyType::BlueGreen => "BLUE_GREEN",
                StrategyType::Linear => "LINEAR",
                StrategyType::Exponential => "EXPONENTIAL",
                StrategyType::Custom => "CUSTOM",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "BLUE_GREEN" => Some(Self::BlueGreen),
                "LINEAR" => Some(Self::Linear),
                "EXPONENTIAL" => Some(Self::Exponential),
                "CUSTOM" => Some(Self::Custom),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Rollout {
        #[prost(message, tag = "2")]
        BlueGreen(super::BlueGreenStrategy),
        #[prost(message, tag = "3")]
        Linear(super::LinearStrategy),
        #[prost(message, tag = "4")]
        Exponential(super::ExponentialStrategy),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlueGreenStrategy {
    #[prost(bool, tag = "1")]
    pub blue_green_field: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LinearStrategy {
    #[prost(int32, tag = "1")]
    pub steps: i32,
    #[prost(int32, tag = "2")]
    pub interval_seconds: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExponentialStrategy {
    #[prost(int32, tag = "1")]
    pub initial_percentage: i32,
    #[prost(int32, tag = "2")]
    pub steps: i32,
    #[prost(int32, tag = "3")]
    pub interval_seconds: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateStrategyRequest {
    #[prost(message, optional, tag = "1")]
    pub strategy: ::core::option::Option<RolloutStrategy>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateStrategyResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStrategyRequest {
    #[prost(string, tag = "1")]
    pub strategy_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStrategyResponse {
    #[prost(message, optional, tag = "1")]
    pub strategy: ::core::option::Option<RolloutStrategy>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListStrategiesRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListStrategiesResponse {
    #[prost(message, repeated, tag = "1")]
    pub strategies: ::prost::alloc::vec::Vec<RolloutStrategy>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateStrategyRequest {
    #[prost(string, tag = "1")]
    pub strategy_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub strategy: ::core::option::Option<RolloutStrategy>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateStrategyResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteStrategyRequest {
    #[prost(string, tag = "1")]
    pub strategy_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteStrategyResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartRolloutRequest {
    #[prost(message, optional, tag = "1")]
    pub strategy: ::core::option::Option<RolloutStrategy>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartRolloutResponse {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
/// Generated client implementations.
pub mod rollout_strategy_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct RolloutStrategyServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RolloutStrategyServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> RolloutStrategyServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> RolloutStrategyServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            RolloutStrategyServiceClient::new(
                InterceptedService::new(inner, interceptor),
            )
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn create_strategy(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateStrategyRequest>,
        ) -> Result<tonic::Response<super::CreateStrategyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/rollout.RolloutStrategyService/CreateStrategy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_strategy(
            &mut self,
            request: impl tonic::IntoRequest<super::GetStrategyRequest>,
        ) -> Result<tonic::Response<super::GetStrategyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/rollout.RolloutStrategyService/GetStrategy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_strategies(
            &mut self,
            request: impl tonic::IntoRequest<super::ListStrategiesRequest>,
        ) -> Result<tonic::Response<super::ListStrategiesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/rollout.RolloutStrategyService/ListStrategies",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_strategy(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateStrategyRequest>,
        ) -> Result<tonic::Response<super::UpdateStrategyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/rollout.RolloutStrategyService/UpdateStrategy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_strategy(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteStrategyRequest>,
        ) -> Result<tonic::Response<super::DeleteStrategyResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/rollout.RolloutStrategyService/DeleteStrategy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn start_rollout(
            &mut self,
            request: impl tonic::IntoRequest<super::StartRolloutRequest>,
        ) -> Result<tonic::Response<super::StartRolloutResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/rollout.RolloutStrategyService/StartRollout",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod rollout_strategy_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with RolloutStrategyServiceServer.
    #[async_trait]
    pub trait RolloutStrategyService: Send + Sync + 'static {
        async fn create_strategy(
            &self,
            request: tonic::Request<super::CreateStrategyRequest>,
        ) -> Result<tonic::Response<super::CreateStrategyResponse>, tonic::Status>;
        async fn get_strategy(
            &self,
            request: tonic::Request<super::GetStrategyRequest>,
        ) -> Result<tonic::Response<super::GetStrategyResponse>, tonic::Status>;
        async fn list_strategies(
            &self,
            request: tonic::Request<super::ListStrategiesRequest>,
        ) -> Result<tonic::Response<super::ListStrategiesResponse>, tonic::Status>;
        async fn update_strategy(
            &self,
            request: tonic::Request<super::UpdateStrategyRequest>,
        ) -> Result<tonic::Response<super::UpdateStrategyResponse>, tonic::Status>;
        async fn delete_strategy(
            &self,
            request: tonic::Request<super::DeleteStrategyRequest>,
        ) -> Result<tonic::Response<super::DeleteStrategyResponse>, tonic::Status>;
        async fn start_rollout(
            &self,
            request: tonic::Request<super::StartRolloutRequest>,
        ) -> Result<tonic::Response<super::StartRolloutResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct RolloutStrategyServiceServer<T: RolloutStrategyService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: RolloutStrategyService> RolloutStrategyServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for RolloutStrategyServiceServer<T>
    where
        T: RolloutStrategyService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/rollout.RolloutStrategyService/CreateStrategy" => {
                    #[allow(non_camel_case_types)]
                    struct CreateStrategySvc<T: RolloutStrategyService>(pub Arc<T>);
                    impl<
                        T: RolloutStrategyService,
                    > tonic::server::UnaryService<super::CreateStrategyRequest>
                    for CreateStrategySvc<T> {
                        type Response = super::CreateStrategyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateStrategyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_strategy(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateStrategySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/rollout.RolloutStrategyService/GetStrategy" => {
                    #[allow(non_camel_case_types)]
                    struct GetStrategySvc<T: RolloutStrategyService>(pub Arc<T>);
                    impl<
                        T: RolloutStrategyService,
                    > tonic::server::UnaryService<super::GetStrategyRequest>
                    for GetStrategySvc<T> {
                        type Response = super::GetStrategyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetStrategyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_strategy(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetStrategySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/rollout.RolloutStrategyService/ListStrategies" => {
                    #[allow(non_camel_case_types)]
                    struct ListStrategiesSvc<T: RolloutStrategyService>(pub Arc<T>);
                    impl<
                        T: RolloutStrategyService,
                    > tonic::server::UnaryService<super::ListStrategiesRequest>
                    for ListStrategiesSvc<T> {
                        type Response = super::ListStrategiesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListStrategiesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).list_strategies(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ListStrategiesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/rollout.RolloutStrategyService/UpdateStrategy" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateStrategySvc<T: RolloutStrategyService>(pub Arc<T>);
                    impl<
                        T: RolloutStrategyService,
                    > tonic::server::UnaryService<super::UpdateStrategyRequest>
                    for UpdateStrategySvc<T> {
                        type Response = super::UpdateStrategyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateStrategyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).update_strategy(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateStrategySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/rollout.RolloutStrategyService/DeleteStrategy" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteStrategySvc<T: RolloutStrategyService>(pub Arc<T>);
                    impl<
                        T: RolloutStrategyService,
                    > tonic::server::UnaryService<super::DeleteStrategyRequest>
                    for DeleteStrategySvc<T> {
                        type Response = super::DeleteStrategyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteStrategyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).delete_strategy(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteStrategySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/rollout.RolloutStrategyService/StartRollout" => {
                    #[allow(non_camel_case_types)]
                    struct StartRolloutSvc<T: RolloutStrategyService>(pub Arc<T>);
                    impl<
                        T: RolloutStrategyService,
                    > tonic::server::UnaryService<super::StartRolloutRequest>
                    for StartRolloutSvc<T> {
                        type Response = super::StartRolloutResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StartRolloutRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).start_rollout(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StartRolloutSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: RolloutStrategyService> Clone for RolloutStrategyServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: RolloutStrategyService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: RolloutStrategyService> tonic::server::NamedService
    for RolloutStrategyServiceServer<T> {
        const NAME: &'static str = "rollout.RolloutStrategyService";
    }
}
