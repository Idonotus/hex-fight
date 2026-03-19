use std::marker::PhantomData;

use crate::engine::{
    cards::{AssignedBand, BaseBand, CardValue, Stacks},
    colors::{Color, ColorComparison},
};

#[derive(Copy, Clone)]
pub struct AllColorPlugin {}
pub struct AllColorBand {
    numeral: u8,
    plugin: AllColorPlugin,
}

impl AllColorBand {
    pub fn new(n: u8) -> Self {
        Self {
            numeral: n,
            plugin: AllColorPlugin {},
        }
    }

    pub fn get_plugin(&self) -> AllColorPlugin {
        return self.plugin;
    }

    pub fn generate_card(&mut self, c_id: u64) -> SimpleCard {
        let (c_id, r) = (c_id / 256, (c_id % 256).try_into().unwrap());
        let (c_id, g) = (c_id / 256, (c_id % 256).try_into().unwrap());
        let (value, b) = (
            (c_id / 256).try_into().unwrap(),
            (c_id % 256).try_into().unwrap(),
        );

        return SimpleCard::new(Color { r, g, b }, value);
    }
}

impl BaseBand for AllColorBand {
    fn get_band_size(&self) -> u64 {
        let i: u64 = self.numeral.into();
        return 0x1000000u64 * i;
    }
}

pub struct SimpleCard {
    color: Color,
    value: CardValue,
}

impl SimpleCard {
    pub fn new(color: Color, value: u8) -> Self {
        Self {
            color,
            value: CardValue::Numeral(value),
        }
    }
}

impl Stacks for SimpleCard {
    fn get_value(&self) -> CardValue {
        return self.value;
    }

    fn get_color(&self) -> Option<Color> {
        return Some(self.color);
    }

    fn get_stacking_priority(&self) -> i16 {
        0
    }
}

pub struct ChooseColorCard {
    color: Option<Color>,
    value: CardValue,
}

pub struct PluralBand<'a, Band, C>(pub Band, u64, PhantomData<&'a C>)
where
    Band: AssignedBand<'a, C>;

impl<'a, Band, C> PluralBand<'a, Band, C>
where
    Band: AssignedBand<'a, C>,
{
    fn new(band: Band, amount: u64) -> Self {
        Self(band, amount, PhantomData)
    }
}

impl<'a, Band: AssignedBand<'a, C>, C> BaseBand for PluralBand<'a, Band, C> {
    fn get_band_size(&self) -> u64 {
        self.0.get_band_size() * self.1
    }
}

impl<'a, Band: AssignedBand<'a, C>, C> AssignedBand<'a, C> for PluralBand<'a, Band, C> {
    fn generate_card(&mut self, c_id: u64) -> C {
        self.0.generate_card(c_id / self.1)
    }
}
