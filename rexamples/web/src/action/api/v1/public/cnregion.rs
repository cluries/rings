pub(crate) mod action {
    use crate::action::api::v1::public::cnregion::{request, responses};
    use rings::{
        axum::{debug_handler, extract::Query, Extension},
        web::{api::Out, context::Context},
    };

    use rings::service::ServiceTrait;

    pub type OutRegion = Out<responses::Region>;

    pub async fn point(Extension(_context): Extension<Context>, Query(query): Query<request::GEOPoint>) -> OutRegion {
        let mut region: responses::Region = Default::default();

        region.id = (1000.0 * (query.lng + query.lat)) as i64;

        Out::ok(region)
    }

    #[debug_handler]
    pub async fn childrens(Extension(_context): Extension<Context>) -> Out<responses::Region> {
        use service::public::cnregion::CNRRegion;

        let shared = rings::service::ServiceManager::shared().await;

        let maci = rings::with_service_write!(shared, CNRRegion, dos, {
            let c = dos.rnd(100);
            c + 20
        })
        .unwrap();

        let i = shared
            .using_mut::<CNRRegion, _, _>(|region| {
                let v = region.rnd(100);
                async move { v + 100 }
            })
            .unwrap()
            .await;

        rings::tracing::info!("---------1-----------{} - {}", maci, i);
        //
        // let _ = shared.using_mut::<CNRRegion, _, _>(|_region| async {
        //     rings::tracing::error!("CNRRegion API call complete, using mut block 2");
        // }).await;

        rings::tracing::info!("---------2------------");

        rings::erx::Erx::new("abc").into()
    }
}

pub(crate) mod request {
    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
    #[serde(crate = "rings::serde")]
    pub(crate) struct GEOPoint {
        pub lat: f64, // latitude
        pub lng: f64, //longitude
    }
}

pub(crate) mod responses {
    #[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
    #[serde(crate = "rings::serde")]
    pub(crate) struct Region {
        pub id: i64,
        pub parent: i64,
        pub level: i32,
        pub name: String,
        pub code: String,
        pub spelling: String,
        pub partial: String,
        pub children: Vec<Region>,
    }
}
