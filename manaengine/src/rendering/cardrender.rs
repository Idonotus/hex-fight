use std::fmt::Display;

use bevy::prelude::*;

use crate::rendering::assetinterface::ExpectedAssetRef;

use super::{
    super::cards::{AssignedBand, BandSet},
    assetinterface::{Asset, AssetRequest, MaterialCache},
};

// pub struct Details {
//     pub name: String,
//     pub description: String,
// }

// impl Display for Details {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}\n{}", self.name, self.description)
//     }
// }

pub trait Assetable {
    // fn get_details(&self) -> Details;
    fn generate_layers(
        &self,
        world: &mut World,
        card_size: Rectangle,
        base_entity: Entity,
        assets: Vec<Asset>,
    ) -> ();
    fn request_assets<'a>(&self, context: &'a mut dyn AssetRequest) -> ();
    fn get_asset_count(&self) -> usize;
}

pub trait AssetableGroup {
    fn asset_expectations(&self) -> Vec<ExpectedAssetRef>;
    fn request_assets(&self) -> Vec<String>;
    fn predict_material(&self, cache: &mut MaterialCache) -> ();
}

impl<'a, Band, C> AssetableGroup for BandSet<'a, Band, C>
where
    Band: AssetableGroup + AssignedBand<'a, C>,
{
    fn asset_expectations(&self) -> Vec<ExpectedAssetRef> {
        self.iter().flat_map(|a| a.asset_expectations()).collect()
    }

    fn request_assets(&self) -> Vec<String> {
        self.iter().flat_map(|a| a.request_assets()).collect()
    }

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
        for band in self.iter() {
            band.predict_material(cache)
        }
    }
}
