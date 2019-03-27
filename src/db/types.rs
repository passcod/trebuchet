use diesel_derive_enum::DbEnum;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, DbEnum, Debug, Deserialize, Serialize)]
#[DieselType = "Release_state"]
pub enum ReleaseState {
    Todo,
    Building,
    Ready,
}
