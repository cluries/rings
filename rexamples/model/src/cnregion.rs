use rings::{
    erx,
    model::rs::{Many, One},
    use_seaorm_min,
};

use crate::model::db;

use_seaorm_min!();

ringm::seaorm_mo!(Cnregion, cn_region);

impl CnregionRetriever {
    pub async fn get() -> One<CnregionMde> {
        CnregionEnt::find_by_id(1).one(db()?).await.map_err(erx::smp)
    }

    pub async fn gets(pks: Vec<i64>) -> Many<CnregionMde> {
        CnregionEnt::find().filter(CnregionCol::Id.is_in(pks)).all(db()?).await.map_err(erx::smp)
    }
}

impl CnregionRepository {}
