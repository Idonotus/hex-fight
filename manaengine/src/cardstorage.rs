use std::{mem::swap, ops::Range};

pub struct LiveCardId(usize);
pub struct LLHRef(pub usize);

enum LiveCardSlot<T> {
    Empty(Option<usize>),
    Filled(T),
}

pub struct LiveCardStorage<T> {
    virtual_map: Vec<(u64, usize)>,
    live_storage: Vec<LiveCardSlot<T>>,
    head: Option<usize>,
}

struct LiveCardAllocator<'a, T> {
    storage: &'a mut LiveCardStorage<T>,
    slots: Vec<usize>,
}

trait CardWrite<T> {
    fn write_location(&mut self, location: LiveCardId, card: T) -> Result<LiveCardId, T>;
    fn allocate(&mut self, card: T) -> Result<LiveCardId, T> {
        let Some(id) = self.get_empty_location() else {
            return Err(card);
        };
        return self.write_location(id, card);
    }
    fn get_empty_location(&self) -> Option<LiveCardId>;
    fn is_full(&self) -> bool;
}

impl<T> CardWrite<T> for LiveCardStorage<T> {
    fn write_location(&mut self, location: LiveCardId, card: T) -> Result<LiveCardId, T> {
        let Some(l) = self.head else {
            return Err(card);
        };
        if l != location.0 {
            return Err(card);
        }
        let mut s = LiveCardSlot::Filled(card);
        swap(&mut self.live_storage[location.0], &mut s);
        let LiveCardSlot::Empty(n) = s else {
            panic!();
        };
        self.head = n;
        return Ok(location);
    }
    fn get_empty_location(&self) -> Option<LiveCardId> {
        return self.head.map(|i| LiveCardId(i));
    }
    fn is_full(&self) -> bool {
        self.head == None
    }
}

impl<'a, T> CardWrite<T> for LiveCardAllocator<'a, T> {
    fn write_location(&mut self, location: LiveCardId, card: T) -> Result<LiveCardId, T> {
        self.slots.pop();
        self.storage.write_location(location, card)
    }
    fn get_empty_location(&self) -> Option<LiveCardId> {
        self.slots.get(self.slots.len() - 1).map(|i| LiveCardId(*i))
    }
    fn is_full(&self) -> bool {
        self.slots.len() == 0
    }
}

trait CardStorage<T>: CardWrite<T> {
    fn exists(&self, location: &LiveCardId) -> bool;
    fn drop_live_card(&mut self, location: LiveCardId) -> Option<T>;
}

impl<T> CardStorage<T> for LiveCardStorage<T> {
    fn exists(&self, location: &LiveCardId) -> bool {
        match self.live_storage[location.0] {
            LiveCardSlot::Empty(_) => false,
            LiveCardSlot::Filled(_) => true,
        }
    }

    fn drop_live_card(&mut self, location: LiveCardId) -> Option<T> {
        if !self.exists(&location) {
            return None;
        }
        let mut nhead = Some(location.0);
        swap(&mut nhead, &mut self.head);
        let mut s = LiveCardSlot::Empty(nhead);
        swap(&mut s, &mut self.live_storage[location.0]);
        return match s {
            LiveCardSlot::Empty(_) => None, // Shouldn't happen but w\e
            LiveCardSlot::Filled(d) => Some(d),
        };
    }
}

// Overly generic? Yeah
// Does it work? Idk
struct LHead {
    pointer: Option<usize>,
    size: usize,
    cap: Option<usize>,
}

impl LHead {
    fn new(cap: Option<usize>) -> Self {
        Self {
            pointer: None,
            size: 0,
            cap,
        }
    }
}

struct LItem<T> {
    prev: Option<usize>,
    next: Option<usize>,
    pub data: T,
    parent: usize,
}

impl<T> LItem<T> {
    fn new(data: T) -> Self {
        Self {
            prev: None,
            next: None,
            data,
            parent: 0,
        }
    }
}

pub struct LL<T> {
    data: Vec<LItem<T>>,
    heads: Vec<LHead>,
}

impl<T> LL<T> {
    const INTERNAL_NULL_HEAD: usize = 0;
    pub fn new(headsdata: Vec<Option<usize>>) -> Self {
        let mut hself = LL {
            data: Vec::new(),
            heads: headsdata.into_iter().map(LHead::new).collect(),
        };
        hself.heads.insert(0, LHead::new(None));
        return hself;
    }

    fn expand(&mut self, amount: usize) -> Range<usize>
    where
        T: Default,
    {
        let plen = self.data.len();
        self.data
            .append(&mut (0..amount).map(|_| LItem::<T>::new(T::default())).collect());
        return plen..(plen + amount);
    }

    fn expand_with_val(&mut self, amount: usize, def_val: T) -> Range<usize>
    where
        T: Clone,
    {
        let plen = self.data.len();
        self.data.append(
            &mut (0..amount)
                .map(|_| LItem::<T>::new(def_val.clone()))
                .collect(),
        );
        return plen..(plen + amount);
    }

    fn is_full(&self) -> bool {
        self.heads[Self::INTERNAL_NULL_HEAD].size == 0
    }

    fn is_head_full(&self, head: usize) -> bool {
        self.heads[head + 1].size == 0
    }

    fn push_to_head(&mut self, head: Option<usize>, item: usize) {
        let headidx = Self::convert_head(head);
        let mut t = Some(item);
        swap(&mut t, &mut self.heads[headidx].pointer);

        self.data[item].parent = headidx;
        swap(&mut t, &mut self.data[item].next);
    }

    fn detach_item(&mut self, item: usize) {
        let prev = self.data[item].prev.take();
        let next = self.data[item].next.take();
        if let Some(idx) = prev {
            self.data[idx].next = next;
        } else {
            let h = self.data[item].parent;
            swap(&mut self.heads[h].pointer, &mut next.clone());
        }
        if let Some(idx) = next {
            self.data[idx].prev = prev;
        }
    }

    fn insert_item(&mut self, item: usize, insert_after: usize) {
        let mut insert_before = Some(item);
        swap(&mut self.data[insert_after].next, &mut insert_before);
        self.data[item].prev = Some(insert_after);
        self.data[item].next = insert_before;
        if let Some(idx) = insert_before {
            self.data[idx].prev = Some(item);
        }
    }

    pub fn follow(&self, head: Option<usize>) -> LFollower {
        LFollower::new(self.heads[Self::convert_head(head)].pointer)
    }

    fn convert_head(head: Option<usize>) -> usize {
        match head {
            Some(i) => i + 1,
            None => 0,
        }
    }

    fn get(&self, location: usize) -> &LItem<T> {
        &self.data[location]
    }

    fn get_mut(&mut self, location: usize) -> &mut LItem<T> {
        &mut self.data[location]
    }
}

pub struct LFollower {
    cur_pos: Option<usize>,
}

impl LFollower {
    pub fn new(cur_pos: Option<usize>) -> Self {
        Self { cur_pos }
    }

    pub fn next<T>(&mut self, map: &LL<T>) -> bool {
        match self.cur_pos {
            None => false,
            Some(i) => {
                self.cur_pos = map.data[i].next.clone();
                true
            }
        }
    }

    pub fn position(&self) -> &Option<usize> {
        &self.cur_pos
    }

    pub fn get<'a, T>(&self, map: &'a LL<T>) -> &'a T {
        &map.get(self.cur_pos.unwrap()).data
    }

    pub fn get_mut<'a, T>(&self, map: &'a mut LL<T>) -> &'a mut T {
        &mut map.get_mut(self.cur_pos.unwrap()).data
    }
}
