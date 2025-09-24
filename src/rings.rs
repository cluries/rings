use crate::any::AnyTrait;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tracing::{error, info, span};

/// Rings Application
/// RingsApplication = Arc<RwLock<Rings>>
pub type RingsApplication = Arc<RwLock<Rings>>;

/// Rings
static RINGS: RwLock<Vec<RingsApplication>> = RwLock::new(Vec::new());

static RINGS_INVOKE_MACRO: RwLock<Vec<(String, fn())>> = RwLock::new(Vec::new());

/// name: ringsapp name
pub fn add_rings_invoke_macro(name: &str, func: fn()) {
    RINGS_INVOKE_MACRO.write().unwrap().push((name.to_string(), func));
}

/// Rings Application
pub struct Rings {
    /// Rings Name
    name: String,
    /// Rings Mods
    mods: Vec<Box<dyn RingsMod>>,
    /// Rings State
    state: Arc<RwLock<RingState>>,
    /// Moments
    moments: Vec<Moment>,
}

/// Moment is a moment in time.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Moment {
    name: String,
    time: i64,
}

///
impl Moment {
    /// Moment with current time
    pub fn now(name: &str) -> Self {
        Self { name: name.to_string(), time: chrono::Utc::now().timestamp_micros() }
    }
}

/// Rings State
/// RingState::Init => 1,
/// RingState::Ready => 10,
/// RingState::Working => 100,
/// RingState::Paused => 9999,
/// RingState::Terminating => -10,
/// RingState::Terminated => -1,
/// RingState::Unknown => 0,
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum RingState {
    Init,
    Ready,
    Working,
    Paused,
    Terminating,
    Terminated,
    Unknown,
}

/// Ring Thread Safe State
/// SafeRingState = Arc<RwLock<RingState>>
pub type SafeRingState = Arc<RwLock<RingState>>;

impl Into<i32> for RingState {
    fn into(self) -> i32 {
        match self {
            RingState::Init => 1,
            RingState::Ready => 10,
            RingState::Working => 100,
            RingState::Paused => 9999,
            RingState::Terminating => -10,
            RingState::Terminated => -1,
            RingState::Unknown => 0,
        }
    }
}

impl From<i32> for RingState {
    fn from(value: i32) -> Self {
        match value {
            1 => RingState::Init,
            10 => RingState::Ready,
            100 => RingState::Working,
            9999 => RingState::Paused,
            -10 => RingState::Terminating,
            -1 => RingState::Terminated,
            _ => RingState::Unknown,
        }
    }
}

impl FromStr for RingState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init" => Ok(RingState::Init),
            "ready" => Ok(RingState::Ready),
            "working" => Ok(RingState::Working),
            "paused" => Ok(RingState::Paused),
            "terminating" => Ok(RingState::Terminating),
            "terminated" => Ok(RingState::Terminated),
            _ => Err(format!("Unknown ring state: {}", s)),
        }
    }
}

impl Into<&str> for RingState {
    fn into(self) -> &'static str {
        match self {
            RingState::Init => "init",
            RingState::Ready => "ready",
            RingState::Working => "working",
            RingState::Paused => "paused",
            RingState::Terminating => "terminating",
            RingState::Terminated => "terminated",
            RingState::Unknown => "unknown",
        }
    }
}

impl RingState {
    pub fn is_ready_to_terminating(&self) -> bool {
        matches!(self, RingState::Init | RingState::Ready | RingState::Working | RingState::Paused)
    }

    pub fn safe_ring_state_set(rs: &SafeRingState, s: RingState) -> Result<(), crate::erx::Erx> {
        *rs.try_write().map_err(crate::erx::smp)? = s;
        Ok(())
    }

    pub fn safe_ring_state_must_set(rs: &SafeRingState, s: RingState) -> Result<(), crate::erx::Erx> {
        *rs.write().map_err(crate::erx::smp)? = s;
        Ok(())
    }

    pub fn safe_ring_state_get(rs: &SafeRingState) -> Result<RingState, crate::erx::Erx> {
        Ok(rs.try_read().map_err(crate::erx::smp)?.clone())
    }

    pub fn safe_ring_state_must_get(rs: &SafeRingState) -> Result<RingState, crate::erx::Erx> {
        Ok(rs.read().map_err(crate::erx::smp)?.clone())
    }

    pub fn inited_safe_ring_state() -> SafeRingState {
        Arc::new(RwLock::new(RingState::Init))
    }
}

#[async_trait]
pub trait RingsMod: AnyTrait + Send + Sync {
    fn name(&self) -> String;
    fn duplicate_able(&self) -> bool;
    async fn initialize(&mut self) -> Result<(), crate::erx::Erx>;
    async fn unregister(&mut self) -> Result<(), crate::erx::Erx>;
    async fn shutdown(&mut self) -> Result<(), crate::erx::Erx>;
    async fn fire(&mut self) -> Result<(), crate::erx::Erx>;
    fn stage(&self) -> RingState;
    fn level(&self) -> i64;
}

/// R
/// Just like namespace, call some rings methods
pub struct R;
impl R {
    pub fn instance(name: String) -> Result<RingsApplication, String> {
        for ring in RINGS.read().unwrap().iter() {
            let r = Arc::clone(ring);
            if r.read().unwrap().name.eq(&name) {
                return Ok(Arc::clone(ring));
            }
        }

        Err(format!("Rings instance '{}' not found", name))
    }

    // make rings
    pub async fn make(name: &str) -> RingsApplication {
        crate::log::logging_initialize().await;

        let app = Rings {
            name: name.to_string(),
            mods: vec![],
            state: Arc::new(RwLock::new(RingState::Init)),
            moments: vec![Moment::now("make")],
        };

        // app.register_mod(SchedulerManager::new()).await;

        let arc: RingsApplication = Arc::new(RwLock::new(app));

        match RINGS.try_write() {
            Ok(mut rings) => {
                // # TODO
                // Currently, only one app registration is supported.
                // When we change this later, we need to synchronously modify
                // the support for multiple RingApps in other components
                // like ServiceManager, SchedulerManager, and Model.
                if rings.len() > 1 {
                    panic!(
                        "Sorry, you've already registered an app. \
                    The current version only supports registering one app. \
                    We'll support multiple apps as soon as possible."
                    );
                }

                rings.push(Arc::clone(&arc));
            },
            Err(ex) => {
                error!("make rings push RINGS: {}", ex);
            },
        }

        info!("rings application:{} made", name);

        let invoke_macros = RINGS_INVOKE_MACRO.read().expect("unable to read RINGS_INVOKE_MACRO");
        for (ring_app_name, func) in invoke_macros.iter() {
            if name.eq(ring_app_name) {
                func();
            }
        }

        Arc::clone(&arc)
    }

    pub async fn perform(rings_app: &RingsApplication) {
        match rings_app.try_write() {
            Ok(mut guard) => {
                guard.fire().await;
            },
            Err(ex) => {
                error!("{}", ex);
            },
        };

        Rings::serve(rings_app).await;
    }
}

impl Rings {
    pub fn make_moment(&mut self, name: &str) {
        self.moments.push(Moment::now(name));
    }

    pub fn get_moments(&self, pred: Option<String>, after: Option<i64>) -> Vec<Moment> {
        let mut moments: Vec<Moment> = self.moments.clone();
        if let Some(pred) = pred {
            moments.retain(|m| m.name.contains(&pred));
        }
        if let Some(after) = after {
            moments.retain(|m| m.time >= after);
        }
        moments
    }

    pub async fn register_mod<T: RingsMod>(&mut self, mut md: T) -> &mut Self {
        if !md.duplicate_able() && self.mods.iter().any(|x| x.name().eq(&md.name())) {
            error!("Mod '{}' already registered!", md.name());
            return self;
        }

        md.initialize().await.expect("initialize mod failed.");
        self.mods.push(Box::new(md));
        self.moments.push(Moment::now(&format!("mod [{}] registered", &self.name)));

        // self.mods.sort_by(|a, b| a.level().cmp(&b.level()));
        self.mods.sort_by_key(|a| a.level());

        self
    }

    pub async fn shutdown(&mut self) {
        if !self.state.read().unwrap().is_ready_to_terminating() {
            return;
        }

        self.make_moment("shutdown");

        info!("rings::shutdown....");
        *self.state.write().unwrap() = RingState::Terminating;

        for md in self.mods.iter_mut() {
            match md.shutdown().await {
                Ok(_) => {
                    info!("rings mod:[ {} ] shutdown accepted", md.name());
                },
                Err(ex) => {
                    error!("failed to signal shutdown: {} error: {}", md.name(), ex.message());
                },
            }
        }
    }

    pub fn get_mod<T: RingsMod>(&self, name: &str) -> Option<&T> {
        for m in &self.mods {
            if m.name().eq(name) {
                return Some(m.as_any().downcast_ref::<T>().unwrap());
            }
        }

        None
    }

    pub fn get_mod_mut<T: RingsMod>(&mut self, name: &str) -> Option<&mut T> {
        for m in &mut self.mods {
            if m.name().eq(name) {
                return Some(m.as_any_mut().downcast_mut::<T>().unwrap());
            }
        }

        None
    }

    pub async fn remove_mod(&mut self, name: &str) -> &mut Self {
        let drain = |m: &Box<dyn RingsMod>| m.name().eq(name);

        for m in &mut self.mods {
            if drain(m) {
                m.unregister().await.expect("unregister mod failed.");
            }
        }

        self.mods.retain(|m| !drain(m));

        self
    }

    pub fn get_state(&self) -> Result<RingState, crate::erx::Erx> {
        Ok(self.state.try_read().map_err(crate::erx::smp)?.clone())
    }

    pub fn get_state_unchecked(&self) -> RingState {
        match self.state.try_read() {
            Ok(state) => state.clone(),
            Err(_) => RingState::Unknown,
        }
    }

    pub async fn fire(&mut self) {
        let span = span!(tracing::Level::INFO, "FireMod");
        let _guard = span.enter();

        info!("Fire Rings, Mods: {}", self.mods.len());

        for m in self.mods.iter_mut() {
            if let Err(e) = m.fire().await {
                error!("fire level {} mod:[ {} ] error:{}", m.level(), m.name(), e.message());
            } else {
                info!("fire level {} mod:[ {} ] success.", m.level(), m.name());
            }
        }

        *self.state.write().unwrap() = RingState::Working;

        // let mut groups: HashMap<i64, Vec<_>> = HashMap::new();
        // for m in self.mods.iter_mut() {
        //     groups.entry(m.level()).or_default().push(m.fire());
        // }
        //
        // let mut levels: Vec<i64> = groups.keys().cloned().collect();
        // for level in levels {
        //     let futures = groups.get(&level).unwrap();
        //     if !futures.is_empty() {
        //         let fu = futures.iter().map(|x| *x).collect::<Vec<_>>();
        //         futures_util::future::join_all(fu).await;
        //     }
        // }

        // let mut groups = vec![];
        // for m in self.mods.iter_mut() {
        //     groups.push((m.level(), m.fire()));
        //
        //     let m = (m.level(), m.fire());
        //
        //
        //     let s = m.fire();
        //     info!("fire mod:[ {} ]", m.name());
        //
        //     match m.fire().await {
        //         Err(ex) => {
        //             error!("failed to fire mod: {} error: {}", m.name(), ex.message());
        //         }
        //         _ => {}
        //     }
        // }

        // let groups: Vec<(i64, _)> = self.mods.iter_mut().map(|m| (m.level(), m.fire())).collect();

        // let ctrl_c = |name: String, stage: Arc<RwLock<RingState>>| async move {
        //     tokio::signal::ctrl_c().await.expect("RINGS attempt to terminate immediately");
        //     *stage.write().unwrap() = RingState::Terminating;
        //     tracing::info!("RINGS:{} received shutdown signal, set state: [Terminating]", name);
        // };
        //
        // tokio::spawn(ctrl_c(self.name.clone(), Arc::clone(&self.state)));
    }

    pub fn mods_stages(&self) -> HashMap<String, RingState> {
        self.mods.iter().map(|m| (m.name().to_string(), m.stage())).collect()
    }

    pub fn mods_all_terminated(&self) -> bool {
        for m in self.mods.iter() {
            if m.stage() != RingState::Terminated {
                return false;
            }
        }

        true
    }

    pub fn set_state(&self, state: RingState) {
        *self.state.write().unwrap() = state;
    }

    async fn serve(app: &RingsApplication) {
        let _ = tokio::join!(Self::catch_signal(app), Self::holding(app));
    }

    async fn catch_signal(app: &RingsApplication) {
        info!("Catch signal started");
        let app = app.clone();
        tokio::signal::ctrl_c().await.expect("attempt to terminate immediately");
        info!("rings::listen_signal_kill received Ctrl-C, shutting down");

        match app.write() {
            Ok(mut write_app) => {
                write_app.shutdown().await;
            },
            Err(er) => {
                error!("failed to listen_signal_kill: {}", er);
            },
        };
    }

    async fn holding(app: &RingsApplication) {
        let app = Arc::clone(app);
        const MAX_CONSECUTIVE_FAILURES: i32 = 8;
        let duration = tokio::time::Duration::from_millis(100);
        let mut consecutive_failures = 0;

        loop {
            if consecutive_failures > MAX_CONSECUTIVE_FAILURES {
                error!("Rings::wait lock error count: {}, now break wait", consecutive_failures);
                break;
            }

            tokio::time::sleep(duration).await;

            match app.try_read() {
                Ok(ring) => {
                    consecutive_failures = 0;
                    let stat = ring.get_state_unchecked();
                    if stat != RingState::Terminating && stat != RingState::Terminated {
                        continue;
                    }

                    if ring.mods_all_terminated() {
                        info!("all mods terminated, breaking loop");
                        break;
                    }

                    let mod_stages = ring.mods_stages();
                    info!("mod stages: {:?}", mod_stages);
                },
                Err(_) => {
                    consecutive_failures += 1;
                },
            }
        }

        app.write().unwrap().set_state(RingState::Terminated);
    }

    pub fn description(&self) -> String {
        let mut md = "".to_string();
        md.push_str("[Rings] name:");
        md.push_str(self.name.as_str());
        md.push_str(" mods:");
        for m in &self.mods {
            md.push_str(&m.name());
        }
        md
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_rings() {

        // crate::rings::add_rings_invoke_macro("test_mod")
    }
}
