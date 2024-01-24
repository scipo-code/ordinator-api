use actix::Addr;
use std::collections::HashMap;
use std::sync::Mutex;

struct ActorRegistry {
    registry: Mutex<HashMap<String, Addr<dyn Actor>>>,
}

impl ActorRegistry {
    fn new() -> Self {
        ActorRegistry {
            registry: Mutex::new(HashMap::new()),
        }
    }

    fn register_actor(&self, name: String, addr: Addr<dyn <Actor>>) {
        let mut reg = self.registry.lock().unwrap();
        reg.insert(name, addr);
    }

    fn get_actor(&self, name: &str) -> Option<Addr<dyn Actor>> {
        let reg = self.registry.lock().unwrap();
        reg.get(name).cloned()
    }
}