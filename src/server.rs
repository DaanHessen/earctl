use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use tracing::warn;

use crate::{
    bluetooth,
    error::EarError,
    models::ModelBase,
    service::{EarManager, EarSessionHandle},
    types::{
        AncLevel, CustomEq, EarFitResult, EarSide, EnhancedBassState, EqMode, FirmwareInfo,
        GestureSlot, InEarState, LatencyState, LedColorSet, ModelSummary, PersonalizedAncState,
        SerialIdentity, SessionInfo,
    },
};

#[derive(Clone)]
pub struct ApiState {
    pub manager: Arc<EarManager>,
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/api/session", get(get_session).delete(disconnect))
        .route("/api/session/connect", post(connect))
        .route("/api/session/detect", post(detect_serial))
        .route("/api/session/auto-connect", post(auto_connect))
        .route("/api/session/model", post(update_model))
        .route("/api/battery", get(read_battery))
        .route("/api/anc", get(read_anc).post(set_anc))
        .route("/api/eq", get(read_eq).post(set_eq))
        .route("/api/eq/custom", get(get_custom_eq).post(set_custom_eq))
        .route(
            "/api/enhanced-bass",
            get(get_enhanced_bass).post(set_enhanced_bass),
        )
        .route(
            "/api/personalized-anc",
            get(get_personalized_anc).post(set_personalized_anc),
        )
        .route("/api/in-ear", get(read_in_ear).post(set_in_ear))
        .route("/api/latency", get(read_latency).post(set_latency))
        .route("/api/firmware", get(read_firmware))
        .route("/api/ear-fit", get(read_ear_fit).post(start_ear_fit))
        .route("/api/gestures", get(read_gestures).post(set_gesture))
        .route(
            "/api/led-case",
            get(read_led_case_colors).post(set_led_case_colors),
        )
        .route("/api/ring", post(ring_buds))
        .with_state(state)
}

pub async fn serve(state: ApiState, addr: SocketAddr) -> anyhow::Result<()> {
    let app = router(state);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

type ApiResult<T> = Result<Json<T>, ApiError>;

async fn connect(
    State(state): State<ApiState>,
    Json(request): Json<ConnectRequest>,
) -> ApiResult<SessionInfo> {
    let address: bluer::Address = request.address.parse().map_err(|e| ApiError {
        inner: EarError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid Bluetooth address: {}", e),
        )),
    })?;

    let handle = state.manager.connect(address, request.channel).await?;

    if let Some(model) = request.model {
        apply_model_selector(&handle, model).await?;
    }

    Ok(Json(handle.info().await))
}

async fn disconnect(State(state): State<ApiState>) -> ApiResult<serde_json::Value> {
    state.manager.disconnect().await?;
    Ok(Json(serde_json::json!({ "status": "disconnected" })))
}

async fn get_session(State(state): State<ApiState>) -> ApiResult<SessionInfo> {
    let session = state.manager.session().await?;
    Ok(Json(session.info().await))
}

async fn detect_serial(State(state): State<ApiState>) -> ApiResult<SerialIdentity> {
    let session = state.manager.session().await?;
    let identity = session.detect_serial().await?;
    Ok(Json(identity))
}

async fn auto_connect(
    State(state): State<ApiState>,
    Json(request): Json<AutoConnectRequest>,
) -> ApiResult<SessionInfo> {
    let device =
        bluetooth::resolve_connected_device(request.address.clone(), request.name.clone()).await?;
    let channel = if let Some(ch) = request.channel {
        ch
    } else {
        match bluetooth::detect_rfcomm_channel(&device.address).await {
            Ok(ch) => ch,
            Err(err) => {
                warn!(
                    "Failed to detect RFCOMM channel for {}: {}. Falling back to channel {}",
                    device.address,
                    err,
                    default_rfcomm_channel()
                );
                default_rfcomm_channel()
            }
        }
    };

    // Parse Bluetooth address for bluer
    let bt_address: bluer::Address = device.address.parse().map_err(|_| {
        EarError::Detection(format!("invalid Bluetooth address: {}", device.address))
    })?;

    let handle = state.manager.connect(bt_address, channel).await?;
    if let Some(sku) = request.sku {
        let _ = handle.set_model_from_sku(&sku, None).await?;
    }
    Ok(Json(handle.info().await))
}

async fn update_model(
    State(state): State<ApiState>,
    Json(request): Json<ModelSelector>,
) -> ApiResult<ModelSummary> {
    let session = state.manager.session().await?;
    let summary = apply_model_selector(&session, request).await?;
    Ok(Json(summary))
}

async fn read_battery(State(state): State<ApiState>) -> ApiResult<crate::types::BatteryStatus> {
    let session = state.manager.session().await?;
    let status = session.read_battery().await?;
    Ok(Json(status))
}

async fn read_anc(State(state): State<ApiState>) -> ApiResult<AncLevel> {
    let session = state.manager.session().await?;
    let anc = session.read_anc().await?;
    Ok(Json(anc))
}

async fn set_anc(
    State(state): State<ApiState>,
    Json(req): Json<AncRequest>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_anc(req.level).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn read_eq(State(state): State<ApiState>) -> ApiResult<EqMode> {
    let session = state.manager.session().await?;
    let eq = session.read_eq().await?;
    Ok(Json(eq))
}

async fn set_eq(
    State(state): State<ApiState>,
    Json(req): Json<SetEqRequest>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_eq_mode(req.mode).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn get_custom_eq(State(state): State<ApiState>) -> ApiResult<CustomEq> {
    let session = state.manager.session().await?;
    let eq = session.get_custom_eq().await?;
    Ok(Json(eq))
}

async fn set_custom_eq(
    State(state): State<ApiState>,
    Json(req): Json<CustomEq>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_custom_eq(req).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn get_enhanced_bass(State(state): State<ApiState>) -> ApiResult<EnhancedBassState> {
    let session = state.manager.session().await?;
    let state = session.read_enhanced_bass().await?;
    Ok(Json(state))
}

async fn set_enhanced_bass(
    State(state): State<ApiState>,
    Json(req): Json<EnhancedBassState>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_enhanced_bass(req.enabled, req.level).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn get_personalized_anc(State(state): State<ApiState>) -> ApiResult<PersonalizedAncState> {
    let session = state.manager.session().await?;
    let state = session.get_personalized_anc().await?;
    Ok(Json(state))
}

async fn set_personalized_anc(
    State(state): State<ApiState>,
    Json(req): Json<PersonalizedAncState>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_personalized_anc(req.enabled).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn read_in_ear(State(state): State<ApiState>) -> ApiResult<InEarState> {
    let session = state.manager.session().await?;
    let resp = session.read_in_ear().await?;
    Ok(Json(resp))
}

async fn set_in_ear(
    State(state): State<ApiState>,
    Json(req): Json<InEarState>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_in_ear_detection(req.detection_enabled).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn read_latency(State(state): State<ApiState>) -> ApiResult<LatencyState> {
    let session = state.manager.session().await?;
    let resp = session.read_latency().await?;
    Ok(Json(resp))
}

async fn set_latency(
    State(state): State<ApiState>,
    Json(req): Json<LatencyState>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_latency(req.low_latency_enabled).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn read_firmware(State(state): State<ApiState>) -> ApiResult<FirmwareInfo> {
    let session = state.manager.session().await?;
    Ok(Json(session.read_firmware().await?))
}

async fn start_ear_fit(State(state): State<ApiState>) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.launch_ear_fit_test().await?;
    Ok(Json(serde_json::json!({ "status": "started" })))
}

async fn read_ear_fit(State(state): State<ApiState>) -> ApiResult<EarFitResult> {
    let session = state.manager.session().await?;
    Ok(Json(session.read_ear_fit_result().await?))
}

async fn read_gestures(State(state): State<ApiState>) -> ApiResult<Vec<GestureSlot>> {
    let session = state.manager.session().await?;
    Ok(Json(session.read_gestures().await?))
}

async fn set_gesture(
    State(state): State<ApiState>,
    Json(req): Json<GestureSlot>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_gesture(&req).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn read_led_case_colors(State(state): State<ApiState>) -> ApiResult<LedColorSet> {
    let session = state.manager.session().await?;
    Ok(Json(session.read_led_case_colors().await?))
}

async fn set_led_case_colors(
    State(state): State<ApiState>,
    Json(req): Json<LedColorSet>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.set_led_case_colors(&req).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn ring_buds(
    State(state): State<ApiState>,
    Json(req): Json<RingRequest>,
) -> ApiResult<serde_json::Value> {
    let session = state.manager.session().await?;
    session.ring_buds(req.enable, req.side).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

#[derive(Debug, Deserialize)]
struct ConnectRequest {
    address: String,
    #[serde(default = "default_rfcomm_channel")]
    channel: u8,
    #[serde(default)]
    model: Option<ModelSelector>,
}

fn default_rfcomm_channel() -> u8 {
    1
}

#[derive(Debug, Deserialize)]
struct AutoConnectRequest {
    #[serde(default)]
    address: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    channel: Option<u8>,
    #[serde(default)]
    sku: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModelSelector {
    #[serde(default)]
    model_id: Option<String>,
    #[serde(default)]
    sku: Option<String>,
    #[serde(default)]
    base: Option<ModelBase>,
}

#[derive(Debug, Deserialize)]
struct AncRequest {
    level: AncLevel,
}

#[derive(Debug, Deserialize)]
struct SetEqRequest {
    mode: u8,
}

#[derive(Debug, Deserialize)]
struct RingRequest {
    enable: bool,
    #[serde(default)]
    side: Option<EarSide>,
}

#[derive(Debug)]
struct ApiError {
    inner: EarError,
}

impl From<EarError> for ApiError {
    fn from(inner: EarError) -> Self {
        Self { inner }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.inner {
            EarError::NoSession => StatusCode::NOT_FOUND,
            EarError::AlreadyConnected => StatusCode::CONFLICT,
            EarError::Detection(_) => StatusCode::BAD_REQUEST,
            EarError::Unsupported(_) | EarError::UnknownModel => StatusCode::BAD_REQUEST,
            EarError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({
            "error": format!("{}", self.inner),
        });
        (status, Json(body)).into_response()
    }
}

async fn apply_model_selector(
    session: &EarSessionHandle,
    selector: ModelSelector,
) -> Result<ModelSummary, EarError> {
    if let Some(id) = selector.model_id {
        return session.set_model_by_id(&id).await;
    }
    if let Some(sku) = selector.sku {
        return session.set_model_from_sku(&sku, None).await;
    }
    if let Some(base) = selector.base {
        return Ok(session.set_model_base(base).await);
    }
    Err(EarError::UnknownModel)
}
