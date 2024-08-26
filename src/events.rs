use std::collections::LinkedList;
use zkwasm_rest_abi::MERKLE_MAP;

#[derive(Clone)]
pub struct Event {
    pub has_executed_action: u64
}

impl Event {
    fn compact(&self, buf: &mut Vec<u64>) {
        buf.push(self.has_executed_action);
        zkwasm_rust_sdk::dbg!("compact {:?}", buf);
    }

    fn fetch(buf: &mut Vec<u64>) -> Event {
        zkwasm_rust_sdk::dbg!("fetch{:?}", buf);
        let has_executed_action = buf.pop().unwrap();
        Event {
            has_executed_action: has_executed_action
        }
    }
}

pub struct EventQueue {
    pub counter: u64,
    pub progress: u64,
    pub list: std::collections::LinkedList<Event>
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            counter: 0,
            progress: 0,
            list: LinkedList::new(),
        }
    }

    pub fn store(&self) {
        let n = self.list.len();
        let mut v = Vec::with_capacity(n * 5 + 1);
        for e in self.list.iter() {
            e.compact(&mut v);
        }
        v.push(self.counter);
        v.push(self.progress);
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&[0, 0, 0, 0], v.as_slice());
        let root = kvpair.merkle.root.clone();
        zkwasm_rust_sdk::dbg!("root after store: {:?}\n", root);
    }

    pub fn fetch(&mut self) {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let mut data = kvpair.get(&[0, 0, 0, 0]);
        if !data.is_empty() {
            let counter = data.pop().unwrap();
            let progress = data.pop().unwrap();
            let mut list = LinkedList::new();
            while !data.is_empty() {
                list.push_back(Event::fetch(&mut data))
            }
            self.counter = counter;
            self.progress = progress;
            self.list = list;
        }
    }

    pub fn dump(&self) {
        zkwasm_rust_sdk::dbg!("=-=-= dump queue =-=-=\n");
        for m in self.list.iter() {
            let has_executed_action = m.has_executed_action;
            zkwasm_rust_sdk::dbg!("{:?}\n", has_executed_action);
        }
        zkwasm_rust_sdk::dbg!("=-=-= end =-=-=\n");
    }

    pub fn tick(&mut self) {
        self.dump();
        while let Some(_head) = self.list.front_mut() {
            self.progress += 50; 
        }
        self.counter += 1;
        self.progress += 1;
        self.list.pop_front();
    }

    pub fn insert(
      &mut self,
      has_executed_action: u64,
    ) {
        let mut list = LinkedList::new();
        let node = Event {
          has_executed_action: has_executed_action
        };
        list.push_back(node);
        list.append(&mut self.list);
        self.list = list;
    }
}