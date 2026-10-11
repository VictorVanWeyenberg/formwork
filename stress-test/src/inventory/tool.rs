use crate::contentref::ToolRef;
use crate::inventory::item::Weight;
use crate::inventory::wallet::Cost;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    #[serde_as(as = "DisplayFromStr")]
    id: ToolRef,
    name: String,
    description: String,
    cost: Cost,
    weight: Weight,
    tool_category: Option<ToolCategory>,
}

impl Tool {
    pub fn new(
        id: ToolRef,
        name: String,
        description: String,
        cost: Cost,
        weight: Weight,
        tool_category: Option<ToolCategory>,
    ) -> Tool {
        Tool {
            id,
            name,
            description,
            cost,
            weight,
            tool_category,
        }
    }
    pub fn id(&self) -> &ToolRef {
        &self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn cost(&self) -> &Cost {
        &self.cost
    }
    pub fn weight(&self) -> &Weight {
        &self.weight
    }
    pub fn tool_category(&self) -> &Option<ToolCategory> {
        &self.tool_category
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    ArtisanTools,
    GamingSet,
    MusicalInstrument,
    Vehicle(VehicleCategory),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum VehicleCategory {
    Mount,
    Drawn,
    Waterborne,
}
