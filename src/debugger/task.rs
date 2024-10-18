pub struct Task {
    pub pid: u32,
    pub id: u32,
    pub name: String,
    pub running: bool,
    pub exited: bool,
}

impl Task {
    pub fn new(pid: u32, id: u32, name: String) -> Task {
        Task {
            pid: pid,
            id: id,
            name: name,
            running: true,
            exited: false,
        }
    }
}
