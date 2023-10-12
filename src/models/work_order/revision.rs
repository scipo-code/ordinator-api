pub struct Revision {
    pub string: String,
    pub shutdown: bool,
}

impl Revision {
    pub fn new(string: String) -> Self {
        Revision {
            string: string.clone(),
            shutdown: !string.contains("NOSD"),
        }
    }

    pub fn with_shutdown(string: String, shutdown: bool) -> Self {
        Revision {
            string,
            shutdown,
        }
    }
}