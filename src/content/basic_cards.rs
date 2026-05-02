use std::marker::PhantomData;

use crate::engine::{
    cards::{AssignedBand, BaseBand, CardValue, DeckCapacity, DeckId, Stacks},
    colors::Color,
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

    pub fn generate_card(&mut self, card_id: DeckId) -> SimpleCard {
        let c_id = *card_id;
        let (c_id, r) = (c_id / 256, (c_id % 256) as u8);
        let (c_id, g) = (c_id / 256, (c_id % 256) as u8);
        let (value, b) = (
            (c_id / 256).try_into().unwrap(),
            (c_id % 256).try_into().unwrap(),
        );

        return SimpleCard::new(Color { r, g, b }, value);
    }
}

impl BaseBand for AllColorBand {
    fn get_band_size(&self) -> DeckCapacity {
        return DeckCapacity(0x1000000u64 * self.numeral as u64);
    }
}

#[derive(Clone)]
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
    fn get_band_size(&self) -> DeckCapacity {
        let mut cap = self.0.get_band_size();
        cap.0 *= self.1;
        return cap;
    }
}

impl<'a, Band: AssignedBand<'a, C>, C> AssignedBand<'a, C> for PluralBand<'a, Band, C> {
    fn generate_card(&mut self, mut c_id: DeckId) -> C {
        c_id.0 /= self.1;
        self.0.generate_card(c_id)
    }
}
